//! # Telemetria e Diagnósticos (Telemetry)
//!
//! Este módulo coleta métricas e estatísticas do RDS para:
//! - **Debugging**: Identificar problemas em drivers
//! - **Monitoramento**: Dashboard de saúde do sistema
//! - **Auditoria**: Histórico de eventos e falhas
//! - **Performance**: Latências, throughputs, contagens
//!
//! ## Métricas Coletadas:
//! - Uptime do RDS
//! - Drivers ativos/falhos
//! - Uso de memória (Driver Zone + DMA Pool)
//! - Transações pendentes por barramento
//! - Eventos de hotplug
//! - Falhas e recuperações
//!
//! ## Exposição:
//! - Syscall `SYS_RDS_TELEMETRY` retorna snapshot
//! - VFS expõe em `/devices/rds/telemetry` (futuro)

use super::device::DeviceId;
use super::driver::DriverError;
use crate::sync::Spinlock;
use alloc::vec::Vec;

// =============================================================================
// ESTRUTURAS DE MÉTRICAS
// =============================================================================

/// Snapshot das métricas do RDS.
///
/// Esta estrutura pode ser serializada para syscall.
#[derive(Debug, Clone)]
pub struct TelemetrySnapshot {
    /// Tick do timer quando o snapshot foi gerado.
    pub timestamp: u64,

    /// Ticks desde inicialização do RDS.
    pub uptime_ticks: u64,

    // --- Contadores de dispositivos ---
    /// Total de dispositivos registrados.
    pub total_devices: usize,

    /// Dispositivos ativos (com driver funcionando).
    pub active_devices: usize,

    /// Dispositivos órfãos (sem driver).
    pub orphan_devices: usize,

    /// Dispositivos isolados (mortos).
    pub dead_devices: usize,

    // --- Contadores de drivers ---
    /// Total de drivers registrados.
    pub total_drivers: usize,

    // --- Memória ---
    /// Bytes alocados na Driver Zone.
    pub driver_zone_used: usize,

    /// Bytes alocados no DMA Pool.
    pub dma_pool_used: usize,

    // --- Eventos ---
    /// Total de eventos emitidos.
    pub events_emitted: u64,

    /// Total de falhas reportadas.
    pub failures_reported: u64,

    /// Total de recuperações bem-sucedidas.
    pub recoveries_successful: u64,
}

impl Default for TelemetrySnapshot {
    fn default() -> Self {
        Self {
            timestamp: 0,
            uptime_ticks: 0,
            total_devices: 0,
            active_devices: 0,
            orphan_devices: 0,
            dead_devices: 0,
            total_drivers: 0,
            driver_zone_used: 0,
            dma_pool_used: 0,
            events_emitted: 0,
            failures_reported: 0,
            recoveries_successful: 0,
        }
    }
}

/// Registro de um evento de falha.
#[derive(Debug, Clone)]
pub struct FailureLog {
    pub device_id: DeviceId,
    pub error: DriverError,
    pub timestamp: u64,
    pub recovered: bool,
}

// =============================================================================
// ESTADO INTERNO
// =============================================================================

/// Estado do sistema de telemetria.
struct TelemetryState {
    /// Tick de quando o RDS foi inicializado.
    init_tick: u64,

    /// Histórico de falhas (últimas N).
    failure_history: Vec<FailureLog>,

    /// Contador de eventos emitidos.
    events_count: u64,

    /// Contador de falhas totais.
    failures_count: u64,

    /// Contador de recuperações bem-sucedidas.
    recoveries_count: u64,

    /// Flag de inicialização.
    initialized: bool,
}

/// Limite de entradas no histórico de falhas.
const MAX_FAILURE_HISTORY: usize = 100;

/// Instância global.
static TELEMETRY: Spinlock<TelemetryState> = Spinlock::new(TelemetryState {
    init_tick: 0,
    failure_history: Vec::new(),
    events_count: 0,
    failures_count: 0,
    recoveries_count: 0,
    initialized: false,
});

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o sistema de telemetria.
pub fn init() {
    crate::kinfo!("(Telemetry) Inicializando sistema de telemetria...");

    let mut state = TELEMETRY.lock();
    state.initialized = true;
    state.init_tick = 0; // TODO: Obter tick real

    crate::kinfo!("(Telemetry) Sistema pronto");
}

/// Gera um snapshot das métricas atuais.
pub fn get_snapshot() -> TelemetrySnapshot {
    let state = TELEMETRY.lock();

    // Coleta dados dos outros módulos
    let memory_stats = super::memory::get_stats();
    let dma_stats = super::dma::get_stats();
    let health = super::monitor::get_health_summary();

    TelemetrySnapshot {
        timestamp: 0,    // TODO: Timestamp real
        uptime_ticks: 0, // TODO: Calcular uptime
        total_devices: super::device_count(),
        active_devices: health.healthy + health.degraded,
        orphan_devices: 0, // TODO: Contar órfãos
        dead_devices: health.dead,
        total_drivers: super::driver_count(),
        driver_zone_used: memory_stats.total_allocated,
        dma_pool_used: dma_stats.total_allocated,
        events_emitted: state.events_count,
        failures_reported: state.failures_count,
        recoveries_successful: state.recoveries_count,
    }
}

/// Registra que um driver foi pareado com sucesso.
pub fn record_driver_bound(device_id: DeviceId, driver_name: &str) {
    let mut state = TELEMETRY.lock();
    state.events_count += 1;

    crate::kinfo!(
        "(Telemetry) Driver bound: device",
        device_id.0,
        "<->",
        driver_name
    );
}

/// Registra que um dispositivo foi adicionado.
pub fn record_device_added(device_id: DeviceId) {
    let mut state = TELEMETRY.lock();
    state.events_count += 1;

    crate::kinfo!("(Telemetry) Device added:", device_id.0);
}

/// Registra que um dispositivo ficou órfão (sem driver).
pub fn record_device_orphan(device_id: DeviceId) {
    let mut state = TELEMETRY.lock();
    state.events_count += 1;

    crate::kwarn!("(Telemetry) Device orphan:", device_id.0);
}

/// Registra uma falha.
pub fn record_failure(device_id: DeviceId, err: DriverError) {
    let mut state = TELEMETRY.lock();
    state.failures_count += 1;

    // Adiciona ao histórico
    let log = FailureLog {
        device_id,
        error: err,
        timestamp: 0, // TODO: Timestamp real
        recovered: false,
    };

    state.failure_history.push(log);

    // Limita tamanho do histórico
    if state.failure_history.len() > MAX_FAILURE_HISTORY {
        state.failure_history.remove(0);
    }

    crate::kerror!(
        "(Telemetry) Failure recorded: device",
        device_id.0,
        "error",
        err.as_str()
    );
}

/// Registra uma recuperação bem-sucedida.
pub fn record_recovery_success(device_id: DeviceId) {
    let mut state = TELEMETRY.lock();
    state.recoveries_count += 1;

    // Marca última falha deste device como recuperada
    for log in state.failure_history.iter_mut().rev() {
        if log.device_id == device_id && !log.recovered {
            log.recovered = true;
            break;
        }
    }

    crate::kinfo!("(Telemetry) Recovery successful: device", device_id.0);
}

/// Retorna histórico de falhas.
pub fn get_failure_history() -> Vec<FailureLog> {
    TELEMETRY.lock().failure_history.clone()
}

/// Retorna últimas N falhas.
pub fn get_recent_failures(count: usize) -> Vec<FailureLog> {
    let state = TELEMETRY.lock();
    let len = state.failure_history.len();
    let start = len.saturating_sub(count);

    state.failure_history[start..].to_vec()
}

/// Imprime sumário da telemetria no log.
pub fn print_summary() {
    let snapshot = get_snapshot();

    crate::kinfo!("=== RDS Telemetry Summary ===");
    crate::kinfo!(
        "  Devices: total=",
        snapshot.total_devices,
        " active=",
        snapshot.active_devices
    );
    crate::kinfo!("  Drivers:", snapshot.total_drivers);
    crate::kinfo!(
        "  Memory: driver_zone=",
        snapshot.driver_zone_used,
        " dma=",
        snapshot.dma_pool_used
    );
    crate::kinfo!("  Events:", snapshot.events_emitted);
    crate::kinfo!(
        "  Failures:",
        snapshot.failures_reported,
        " recovered:",
        snapshot.recoveries_successful
    );
    crate::kinfo!("=============================");
}
