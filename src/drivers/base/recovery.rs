//! # Sistema de Recuperação de Falhas (Recovery Manager)
//!
//! Este módulo é o **estrategista de crise** do RDS. Quando um driver
//! apresenta problemas, o RecoveryManager decide a melhor ação:
//!
//! ## Escada de Recuperação:
//! 1. **Retry** - Tenta a operação novamente
//! 2. **Rebind** - Desanexa e anexa o driver (software reset)
//! 3. **ResetBus** - Reset físico do hardware
//! 4. **Fallback** - Tenta um driver alternativo
//! 5. **Isolate** - Desativa o dispositivo permanentemente
//!
//! ## Filosofia:
//! > "Um driver com problema não derruba o sistema. Tentamos de tudo
//! >  antes de desistir, e quando desistimos, fazemos de forma segura."
//!
//! ## Comportamento Adaptativo:
//! - Se recursos sobrando → mais tentativas
//! - Se recursos escassos → abandona mais rápido
//! - Erros transientes → mais paciência
//! - Erros críticos → escalação rápida

use super::context::{apply_recovery_policy, RecoveryPolicy};
use super::device::{Device, DeviceId, DeviceState};
use super::driver::DriverError;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES DE RECUPERAÇÃO
// =============================================================================

/// Número máximo de tentativas antes de escalar para próxima ação.
pub const DEFAULT_MAX_RETRIES: u8 = 3;

/// Timeout inicial entre tentativas (em ms).
pub const INITIAL_RETRY_DELAY_MS: u32 = 100;

/// Timeout máximo (backoff exponencial).
pub const MAX_RETRY_DELAY_MS: u32 = 2000;

/// Número de erros críticos antes de isolar.
pub const CRITICAL_ERROR_THRESHOLD: u8 = 3;

// =============================================================================
// AÇÕES DE RECUPERAÇÃO
// =============================================================================

/// Ações possíveis que o RecoveryManager pode tomar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Tenta a operação novamente imediatamente.
    Retry,

    /// Desanexa o driver e tenta anexar novamente.
    /// Limpa estado de software mas não reseta hardware.
    Rebind,

    /// Solicita reset físico do hardware via barramento.
    /// Limpa estado de hardware.
    ResetBus,

    /// Tenta carregar um driver alternativo (fallback).
    Fallback,

    /// Desativa o dispositivo permanentemente.
    /// Última opção quando tudo falha.
    Isolate,
}

impl RecoveryAction {
    /// Retorna nome legível da ação.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Retry => "Retry",
            Self::Rebind => "Rebind",
            Self::ResetBus => "ResetBus",
            Self::Fallback => "Fallback",
            Self::Isolate => "Isolate",
        }
    }
}

// =============================================================================
// REGISTRO DE RECUPERAÇÃO
// =============================================================================

/// Registro de tentativas de recuperação para um dispositivo.
///
/// Mantém histórico para evitar loops infinitos e para
/// decidir quando escalar para ações mais drásticas.
#[derive(Debug, Clone)]
struct RecoveryRecord {
    /// ID do dispositivo.
    device_id: DeviceId,

    /// Número de tentativas de retry.
    retry_count: u8,

    /// Número de tentativas de rebind.
    rebind_count: u8,

    /// Número de resets de bus.
    reset_count: u8,

    /// Número de erros críticos.
    critical_count: u8,

    /// Última ação tomada.
    last_action: RecoveryAction,

    /// Timestamp da última ação.
    last_action_time: u64,

    /// Flag indicando se dispositivo foi isolado.
    isolated: bool,
}

impl RecoveryRecord {
    fn new(device_id: DeviceId) -> Self {
        Self {
            device_id,
            retry_count: 0,
            rebind_count: 0,
            reset_count: 0,
            critical_count: 0,
            last_action: RecoveryAction::Retry,
            last_action_time: 0,
            isolated: false,
        }
    }
}

// =============================================================================
// GERENCIADOR DE RECUPERAÇÃO
// =============================================================================

/// Gerenciador de recuperação de falhas.
///
/// Mantém histórico de tentativas e decide ações apropriadas.
pub struct RecoveryManager {
    /// Registros de recuperação por dispositivo.
    records: Spinlock<Vec<RecoveryRecord>>,

    /// Configuração: máximo de retries antes de escalar.
    max_retries: u8,
}

impl RecoveryManager {
    /// Cria novo RecoveryManager.
    pub const fn new() -> Self {
        Self {
            records: Spinlock::new(Vec::new()),
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }

    /// Ponto de entrada principal para falhas de hardware.
    ///
    /// Decide e executa a melhor estratégia de recuperação.
    pub fn handle_failure(&self, dev_arc: Arc<Spinlock<Device>>, err: DriverError) {
        let id = { dev_arc.lock().id };

        crate::kerror!("(Recovery) Iniciando protocolo para device:", id.0);
        crate::kerror!("(Recovery) Erro:", err.as_str());

        // Decide qual ação tomar
        let action = self.decide_action(id, err);

        crate::kinfo!("(Recovery) Ação decidida:", action.as_str());

        // Executa a ação
        match action {
            RecoveryAction::Retry => {
                self.perform_retry(dev_arc, err);
            }
            RecoveryAction::Rebind => {
                self.perform_rebind(dev_arc);
            }
            RecoveryAction::ResetBus => {
                self.perform_bus_reset(dev_arc);
            }
            RecoveryAction::Fallback => {
                self.perform_fallback(dev_arc);
            }
            RecoveryAction::Isolate => {
                self.perform_isolation(dev_arc);
            }
        }
    }

    /// Decide qual ação tomar baseado no histórico.
    fn decide_action(&self, id: DeviceId, err: DriverError) -> RecoveryAction {
        let mut records = self.records.lock();

        // Busca ou cria registro para este dispositivo
        let record = if let Some(r) = records.iter_mut().find(|r| r.device_id == id) {
            r
        } else {
            records.push(RecoveryRecord::new(id));
            records.last_mut().unwrap()
        };

        // Se já isolado, não faz nada
        if record.isolated {
            return RecoveryAction::Isolate;
        }

        // Conta erros críticos
        if err.is_critical() {
            record.critical_count += 1;
        }

        // Decide baseado no histórico
        let action = if record.critical_count >= CRITICAL_ERROR_THRESHOLD {
            // Muitos erros críticos → isolar
            RecoveryAction::Isolate
        } else if record.reset_count >= 2 {
            // Já tentou reset demais → fallback ou isolar
            if record.rebind_count > 0 {
                RecoveryAction::Fallback
            } else {
                RecoveryAction::Isolate
            }
        } else if record.rebind_count >= self.max_retries as u8 {
            // Rebind não funcionou → reset
            RecoveryAction::ResetBus
        } else if record.retry_count >= self.max_retries as u8 {
            // Retry não funcionou → rebind
            RecoveryAction::Rebind
        } else if err.is_transient() {
            // Erro transiente → retry
            record.retry_count += 1;
            RecoveryAction::Retry
        } else {
            // Erro permanente → rebind direto
            record.rebind_count += 1;
            RecoveryAction::Rebind
        };

        record.last_action = action;
        record.last_action_time = 0; // TODO: Timestamp real

        action
    }

    /// Tenta a operação novamente.
    fn perform_retry(&self, dev_arc: Arc<Spinlock<Device>>, _err: DriverError) {
        let dev = dev_arc.lock();
        crate::kinfo!("(Recovery) Retry para:", dev.name_as_str());

        // TODO: Implementar retry real
        // Por enquanto, apenas loga
        crate::kwarn!("(Recovery) perform_retry() parcialmente implementado");
    }

    /// Desanexa e anexa o driver novamente.
    fn perform_rebind(&self, dev_arc: Arc<Spinlock<Device>>) {
        let mut dev = dev_arc.lock();
        let name = dev.name_as_str();

        crate::kinfo!("(Recovery) Rebind para:", name);

        // 1. Remove driver atual
        if let Some(driver) = dev.driver.take() {
            crate::kinfo!("(Recovery) Removendo driver:", driver.name());
            let _ = driver.remove(&mut dev);
        }

        // 2. Marca como inicializando
        dev.set_state(DeviceState::Initializing);
        dev.clear_errors();

        // 3. Solicita novo pareamento
        // TODO: Chamar DriverManager para tentar parear novamente
        crate::kwarn!("(Recovery) Rebind parcialmente implementado - falta reprobe");
    }

    /// Reseta o hardware fisicamente.
    fn perform_bus_reset(&self, dev_arc: Arc<Spinlock<Device>>) {
        let mut dev = dev_arc.lock();
        let name = dev.name_as_str();
        let bus_type = dev.bus_type;

        crate::kwarn!("(Recovery) RESET DE HARDWARE para:", name);

        // 1. Busca o barramento
        if let Some(bus) = super::bus::find_by_type(bus_type) {
            crate::kinfo!("(Recovery) Resetando via:", bus.name());

            // Reset físico
            let success = bus.reset_device(&mut dev);

            if success {
                crate::kinfo!("(Recovery) Reset bem-sucedido");
                dev.set_state(DeviceState::Initializing);
                dev.clear_errors();

                // Incrementa contador
                let mut records = self.records.lock();
                if let Some(r) = records.iter_mut().find(|r| r.device_id == dev.id) {
                    r.reset_count += 1;
                }
            } else {
                crate::kerror!("(Recovery) Reset falhou!");
            }
        } else {
            crate::kerror!("(Recovery) Barramento não encontrado:", bus_type.as_str());
        }
    }

    /// Tenta carregar driver alternativo.
    fn perform_fallback(&self, dev_arc: Arc<Spinlock<Device>>) {
        let dev = dev_arc.lock();
        let name = dev.name_as_str();
        let dev_type = dev.device_type;

        crate::kwarn!("(Recovery) FALLBACK para:", name);

        // Delega para o sistema de fallback
        drop(dev); // Libera lock antes de chamar outro módulo
        super::fallback::try_fallback(dev_arc, dev_type);
    }

    /// Isola o dispositivo permanentemente.
    fn perform_isolation(&self, dev_arc: Arc<Spinlock<Device>>) {
        let mut dev = dev_arc.lock();
        let name = dev.name_as_str();

        crate::kerror!("(Recovery) ISOLAMENTO PERMANENTE:", name);

        // 1. Remove driver
        if let Some(driver) = dev.driver.take() {
            crate::kinfo!("(Recovery) Removendo driver antes de isolar");
            let _ = driver.remove(&mut dev);
        }

        // 2. Marca como morto
        dev.set_state(DeviceState::Dead);

        // 3. Marca no registro
        let mut records = self.records.lock();
        if let Some(r) = records.iter_mut().find(|r| r.device_id == dev.id) {
            r.isolated = true;
        }

        // 4. Libera recursos
        super::memory::free_all_for_device(dev.id);
        super::dma::free_all_for_device(dev.id);

        // 5. Emite evento
        super::events::emit(super::events::DeviceEvent::Removed(dev.id));

        crate::kerror!("(Recovery) Dispositivo isolado:", name);
    }

    /// Reseta o registro de um dispositivo (após recuperação bem-sucedida).
    pub fn reset_record(&self, id: DeviceId) {
        let mut records = self.records.lock();
        if let Some(r) = records.iter_mut().find(|r| r.device_id == id) {
            r.retry_count = 0;
            r.rebind_count = 0;
            r.reset_count = 0;
            r.critical_count = 0;
            r.isolated = false;
        }
    }

    /// Verifica se um dispositivo está isolado.
    pub fn is_isolated(&self, id: DeviceId) -> bool {
        self.records
            .lock()
            .iter()
            .find(|r| r.device_id == id)
            .map(|r| r.isolated)
            .unwrap_or(false)
    }
}
