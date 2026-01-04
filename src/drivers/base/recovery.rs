//! # Gerenciador de Recuperação (Recovery Layer)
//!
//! O estrategista responsável por restaurar a funcionalidade de hardware "no ar".
//! Quando o `MonitorManager` detecta uma anomalia ou um driver reporta um erro crítico,
//! a Recuperação intervém para evitar que o erro se propague para o resto do kernel.
//!
//! ## Estratégia de Defesa (Escala de Intervenção):
//! 1. **Restart**: Desanexa e anexa o driver novamente (Limpa estado de software).
//! 2. **Reset**: Solicita ao barramento um reset físico do chip (Limpa estado de hardware).
//! 3. **Isolamento**: Se a falha persistir, o dispositivo é marcado como `Dead` e
//!    todos os recursos são liberados para evitar vazamentos.
//!
//! Este módulo é o que impede que um periférico USB com mau contato cause um Crash
//! no sistema inteiro.

use super::device::{Device, DeviceId, DeviceState};
use super::driver::DriverError;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Ações possíveis de recuperação
#[derive(Debug, Clone, Copy)]
pub enum RecoveryAction {
    Retry,    // Tentar a operação novamente
    Rebind,   // Recarregar o driver
    ResetBus, // Resetar o slot físico
    Isolate,  // Desativar permanentemente
}

/// Registro de tentativas de recuperação para evitar loops infinitos
struct RecoveryRecord {
    device_id: DeviceId,
    retry_count: u8,
    last_action: RecoveryAction,
}

pub struct RecoveryManager {
    records: Spinlock<Vec<RecoveryRecord>>,
    max_retries: u8,
}

impl RecoveryManager {
    pub const fn new() -> Self {
        Self {
            records: Spinlock::new(Vec::new()),
            max_retries: 3,
        }
    }

    /// Ponto de entrada para falhas de hardware.
    /// Decide a melhor estratégia baseada no histórico do dispositivo.
    pub fn handle_failure(&self, dev_arc: Arc<Spinlock<Device>>, err: DriverError) {
        let id = { dev_arc.lock().id };

        crate::kerror!(
            "(Recovery) Iniciando protocolo de recuperação para ID:",
            id.0
        );

        let action = self.decide_action(id, err);

        match action {
            RecoveryAction::Retry | RecoveryAction::Rebind => {
                self.perform_rebind(dev_arc);
            }
            RecoveryAction::ResetBus => {
                self.perform_bus_reset(dev_arc);
            }
            RecoveryAction::Isolate => {
                self.perform_isolation(dev_arc);
            }
        }
    }

    fn decide_action(&self, id: DeviceId, _err: DriverError) -> RecoveryAction {
        let mut records = self.records.lock();

        if let Some(record) = records.iter_mut().find(|r| r.device_id == id) {
            record.retry_count += 1;

            if record.retry_count > self.max_retries {
                RecoveryAction::Isolate
            } else if record.retry_count == self.max_retries {
                RecoveryAction::ResetBus
            } else {
                RecoveryAction::Rebind
            }
        } else {
            records.push(RecoveryRecord {
                device_id: id,
                retry_count: 1,
                last_action: RecoveryAction::Rebind,
            });
            RecoveryAction::Rebind
        }
    }

    /// Tenta recarregar o driver (Software Reset)
    fn perform_rebind(&self, dev_arc: Arc<Spinlock<Device>>) {
        let mut dev = dev_arc.lock();
        let name = dev.name_as_str();

        crate::kinfo!("(Recovery) Tentando Re-bind do driver para:", name);

        // 1. Remover driver atual (cleanup)
        if let Some(driver) = dev.driver.clone() {
            let _ = driver.remove(&mut dev);
            dev.driver = None;
        }

        dev.state = DeviceState::Initializing;

        // 2. Notificar o DriverManager para tentar parear novamente
        // super::request_reprobe(dev_arc.clone()); // Placeholder para integração mod.rs
    }

    /// Tenta resetar o hardware fisicamente (Hardware Reset)
    fn perform_bus_reset(&self, dev_arc: Arc<Spinlock<Device>>) {
        let mut dev = dev_arc.lock();
        crate::kwarn!(
            "(Recovery) EXTREMO: Resetando barramento para:",
            dev.name_as_str()
        );

        // Busca o barramento pai e solicita reset
        let bus_type = dev.bus_type;
        if let Some(bus) = super::bus::find_by_type(bus_type) {
            bus.reset_device(&mut dev);
        }

        dev.state = DeviceState::Initializing;
    }

    /// Isola o hardware falho para proteger o sistema
    fn perform_isolation(&self, dev_arc: Arc<Spinlock<Device>>) {
        let mut dev = dev_arc.lock();
        crate::kerror!(
            "(Recovery) ISOLAMENTO: Desativando hardware permanentemente:",
            dev.name_as_str()
        );

        if let Some(driver) = dev.driver.clone() {
            let _ = driver.remove(&mut dev);
            dev.driver = None;
        }

        dev.state = DeviceState::Dead;

        // Emite evento final de morte do hardware
        super::events::emit(super::events::DeviceEvent::Removed(dev.id));
    }
}

// =============================================================================
// ROADMAP DE IMPLEMENTAÇÕES FUTURAS (PLANEJAMENTO)
// =============================================================================
//
// 1. Re-scan de Barramento (Hot-Reset):
//    - Implementar a capacidade de remover o objeto Device e forçar o barramento
//      a detectá-lo novamente do zero.
//
// 2. Análise de Causa Raiz (RCA):
//    - Estudar o código de erro (`DriverError`) para decidir se o erro é
//      transiente (mau contato) ou fatal (queima de componente).
//
// 3. Substituição de Driver:
//    - Caso um driver específico falhe continuamente, tentar carregar um
//      driver "Genérico" ou "Fallback" (ex: Usar VESA se o driver Nvidia falhar).
//
// 4. Quarentena de Recursos:
//    - Se um dispositivo for isolado, marcar seus recursos (IRQ/Portas) como
//      "Em Quarentena" para que nenhum outro driver os use acidentalmente.
//
// 5. Interface de Usuário para Recuperação:
//    - Notificar o usuário via GUI: "O Driver de Vídeo falhou e foi reiniciado com sucesso".
