//! # Monitor de Saúde (Health Monitor)
//!
//! Este módulo é o **watchdog de software** do RDS. Ele observa o
//! comportamento dos drivers e detecta problemas antes que causem
//! pânicos irreversíveis.
//!
//! ## Mecanismos:
//! - **Score de Saúde**: Contador de erros por dispositivo
//! - **Classificação**: Erros críticos vs transientes
//! - **Limites**: Thresholds configurable para escalar ações
//! - **Gatilhos**: Dispara RecoveryManager quando limites atingidos
//!
//! ## Estados de Saúde:
//! ```text
//! Healthy → Degraded → Failing → Dead
//!          (poucos    (muitos    (desativado)
//!           erros)     erros)
//! ```
//!
//! ## Filosofia:
//! > "Detectar problemas cedo permite recuperação suave."

use super::device::DeviceId;
use super::driver::DriverError;
use crate::sync::Spinlock;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES DE LIMITES
// =============================================================================

/// Número de erros comuns antes de marcar como Degraded.
pub const ERROR_THRESHOLD_DEGRADED: u32 = 3;

/// Número de erros comuns antes de marcar como Failing.
pub const ERROR_THRESHOLD_FAILING: u32 = 10;

/// Número de erros críticos antes de marcar como Dead.
pub const CRITICAL_THRESHOLD_DEAD: u32 = 3;

/// Intervalo entre verificações de saúde (em ticks).
pub const HEALTH_CHECK_INTERVAL: u64 = 1000;

// =============================================================================
// ESTADOS DE SAÚDE
// =============================================================================

/// Níveis de degradação operacional de um dispositivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    /// Funcionamento nominal, sem erros recentes.
    /// Este é o estado desejado.
    Healthy,

    /// Erros isolados detectados, mas dispositivo continua operando.
    /// Usuário não precisa ser notificado ainda.
    Degraded,

    /// Erros frequentes. Dispositivo pode parar a qualquer momento.
    /// RecoveryManager já foi acionado.
    Failing,

    /// Falha crítica irremediável.
    /// Dispositivo foi desativado permanentemente.
    Dead,
}

impl HealthState {
    /// Retorna nome legível do estado.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Degraded => "Degraded",
            Self::Failing => "Failing",
            Self::Dead => "Dead",
        }
    }

    /// Verifica se o dispositivo está operacional.
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded | Self::Failing)
    }
}

// =============================================================================
// REGISTRO DE MONITORAMENTO
// =============================================================================

/// Registro individual de saúde de um dispositivo.
#[derive(Debug, Clone)]
pub struct DeviceMonitor {
    /// ID do dispositivo monitorado.
    pub device_id: DeviceId,

    /// Estado atual de saúde.
    pub state: HealthState,

    /// Contador total de erros.
    pub error_count: u32,

    /// Contador de erros críticos.
    pub critical_errors: u32,

    /// Último erro ocorrido.
    pub last_error: Option<DriverError>,

    /// Timestamp do último erro.
    pub last_error_time: u64,

    /// Timestamp da última verificação de saúde.
    pub last_check_time: u64,
}

impl DeviceMonitor {
    /// Cria novo monitor para um dispositivo.
    fn new(device_id: DeviceId) -> Self {
        Self {
            device_id,
            state: HealthState::Healthy,
            error_count: 0,
            critical_errors: 0,
            last_error: None,
            last_error_time: 0,
            last_check_time: 0,
        }
    }
}

// =============================================================================
// GERENCIADOR DE MONITORES
// =============================================================================

/// Lista global de monitores de dispositivos.
static MONITORS: Spinlock<Vec<DeviceMonitor>> = Spinlock::new(Vec::new());

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o subsistema de monitoramento.
pub fn init() {
    crate::kinfo!("(Monitor) Inicializando Health Monitor...");

    *INITIALIZED.lock() = true;

    // TODO: Iniciar thread de verificação periódica
    // Por enquanto, monitoramento é reativo (baseado em erros reportados)

    crate::kinfo!("(Monitor) Sistema pronto (modo reativo)");
}

/// Registra a ocorrência de um erro em um dispositivo.
///
/// Atualiza o score de saúde e pode acionar RecoveryManager.
pub fn record_error(id: DeviceId, err: DriverError) {
    let mut monitors = MONITORS.lock();

    // Busca ou cria monitor para este dispositivo
    let monitor = if let Some(m) = monitors.iter_mut().find(|m| m.device_id == id) {
        m
    } else {
        monitors.push(DeviceMonitor::new(id));
        monitors.last_mut().unwrap()
    };

    // Atualiza estatísticas
    monitor.error_count += 1;
    monitor.last_error = Some(err);
    monitor.last_error_time = 0; // TODO: Timestamp real

    // Classifica o erro
    if err.is_critical() {
        monitor.critical_errors += 1;
        crate::kerror!(
            "(Monitor) Erro CRÍTICO para ID:",
            id.0,
            "total críticos:",
            monitor.critical_errors
        );
    }

    // Lógica de transição de estado
    let old_state = monitor.state;

    if monitor.critical_errors >= CRITICAL_THRESHOLD_DEAD {
        monitor.state = HealthState::Dead;
    } else if monitor.error_count >= ERROR_THRESHOLD_FAILING {
        monitor.state = HealthState::Failing;
    } else if monitor.error_count >= ERROR_THRESHOLD_DEGRADED {
        monitor.state = HealthState::Degraded;
    }

    // Se houve degradação, notifica
    if monitor.state != old_state {
        crate::kwarn!(
            "(Monitor) Saúde alterada para ID:",
            id.0,
            old_state.as_str(),
            "->",
            monitor.state.as_str()
        );

        // Se degradou para Failing ou Dead, aciona recuperação
        if monitor.state == HealthState::Failing || monitor.state == HealthState::Dead {
            // Aciona o sistema de recuperação
            super::report_failure(id, err);

            // Emite evento
            super::events::emit(super::events::DeviceEvent::ErrorDetected(id, err));
        }
    }
}

/// Retorna o estado de saúde atual de um dispositivo.
pub fn get_health(id: DeviceId) -> HealthState {
    MONITORS
        .lock()
        .iter()
        .find(|m| m.device_id == id)
        .map(|m| m.state)
        .unwrap_or(HealthState::Healthy)
}

/// Retorna informações completas de monitoramento.
pub fn get_monitor_info(id: DeviceId) -> Option<DeviceMonitor> {
    MONITORS.lock().iter().find(|m| m.device_id == id).cloned()
}

/// Reseta as estatísticas de um dispositivo.
///
/// Chamado após recuperação bem-sucedida.
pub fn reset_stats(id: DeviceId) {
    let mut monitors = MONITORS.lock();

    if let Some(m) = monitors.iter_mut().find(|m| m.device_id == id) {
        m.state = HealthState::Healthy;
        m.error_count = 0;
        m.critical_errors = 0;
        m.last_error = None;
        crate::kinfo!("(Monitor) Estatísticas resetadas para ID:", id.0);
    }
}

/// Marca dispositivo como morto (isolado).
pub fn mark_dead(id: DeviceId) {
    let mut monitors = MONITORS.lock();

    if let Some(m) = monitors.iter_mut().find(|m| m.device_id == id) {
        m.state = HealthState::Dead;
    } else {
        let mut new_monitor = DeviceMonitor::new(id);
        new_monitor.state = HealthState::Dead;
        monitors.push(new_monitor);
    }

    crate::kerror!("(Monitor) Dispositivo marcado como DEAD:", id.0);
}

/// Retorna lista de todos os dispositivos com problemas.
pub fn get_unhealthy_devices() -> Vec<(DeviceId, HealthState)> {
    MONITORS
        .lock()
        .iter()
        .filter(|m| m.state != HealthState::Healthy)
        .map(|m| (m.device_id, m.state))
        .collect()
}

/// Retorna contagem de dispositivos por estado de saúde.
pub fn get_health_summary() -> HealthSummary {
    let monitors = MONITORS.lock();

    HealthSummary {
        healthy: monitors
            .iter()
            .filter(|m| m.state == HealthState::Healthy)
            .count(),
        degraded: monitors
            .iter()
            .filter(|m| m.state == HealthState::Degraded)
            .count(),
        failing: monitors
            .iter()
            .filter(|m| m.state == HealthState::Failing)
            .count(),
        dead: monitors
            .iter()
            .filter(|m| m.state == HealthState::Dead)
            .count(),
        total: monitors.len(),
    }
}

/// Sumário de saúde do sistema.
#[derive(Debug, Clone)]
pub struct HealthSummary {
    pub healthy: usize,
    pub degraded: usize,
    pub failing: usize,
    pub dead: usize,
    pub total: usize,
}
