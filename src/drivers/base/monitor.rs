//! # Monitor de Saúde (Health Monitor)
//!
//! O "Watchdog" de software que observa o comportamento dos drivers em execução.
//! Garante que o sistema saiba quando um hardware está apresentando falhas intermitentes
//! antes que elas se tornem pânicos de kernel irreversíveis.
//!
//! ## Mecanismos:
//! - **Score de Saúde**: Rastreia a frequência e severidade de erros reportados.
//! - **Estados de Saúde**: Define níveis de degradação (`Healthy` -> `Dead`).
//! - **Gatilhos de Recuperação**: Aciona automaticamente o `RecoveryManager` quando limites são atingidos.
//!
//! Este subsistema permite que o RedstoneOS isoladamente desative uma placa de rede
//! instável para preservar a estabilidade global do sistema.

use super::device::{DeviceId, DeviceState};
use super::driver::DriverError;
use crate::sync::Spinlock;
use alloc::vec::Vec;

/// Níveis de degradação operacional de um dispositivo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    /// Funcionamento nominal, sem erros recentes.
    Healthy,
    /// Erros isolados detectados, mas o dispositivo continua operando.
    Degraded,
    /// Erros frequentes. O dispositivo pode parar de responder a qualquer momento.
    Failing,
    /// Falha crítica irremediável ou desativado por excesso de erros.
    Dead,
}

/// Registro individual de monitoramento de saúde
pub struct DeviceMonitor {
    pub device_id: DeviceId,
    pub state: HealthState,
    pub error_count: u32,
    pub critical_errors: u32,
    pub last_error: Option<DriverError>,
    pub last_error_time: u64,
}

static MONITORS: Spinlock<Vec<DeviceMonitor>> = Spinlock::new(Vec::new());

/// Limite de erros comuns antes de marcar como Failing
const ERROR_THRESHOLD_FAILING: u32 = 10;
/// Limite de erros críticos antes de marcar como Dead
const CRITICAL_THRESHOLD_DEAD: u32 = 3;

/// Inicializa o subsistema de monitoramento.
pub fn init() {
    // Futuramente aqui pode ser iniciada uma thread de varredura periódica (watchdog)
}

/// Registra a ocorrência de um erro em um dispositivo.
/// Atualiza automaticamente o estado de saúde baseado na severidade.
pub fn record_error(id: DeviceId, err: DriverError) {
    let mut monitors = MONITORS.lock();

    let monitor = if let Some(m) = monitors.iter_mut().find(|m| m.device_id == id) {
        m
    } else {
        monitors.push(DeviceMonitor {
            device_id: id,
            state: HealthState::Healthy,
            error_count: 0,
            critical_errors: 0,
            last_error: None,
            last_error_time: 0,
        });
        monitors.last_mut().unwrap()
    };

    // Atualizar estatísticas
    monitor.error_count += 1;
    monitor.last_error = Some(err);

    // Classificar erro
    let is_critical = matches!(
        err,
        DriverError::InitFailed | DriverError::AccessDenied | DriverError::NoMemory
    );

    if is_critical {
        monitor.critical_errors += 1;
    }

    // Lógica de transição de estado
    let old_state = monitor.state;

    if monitor.critical_errors >= CRITICAL_THRESHOLD_DEAD {
        monitor.state = HealthState::Dead;
    } else if monitor.error_count >= ERROR_THRESHOLD_FAILING {
        monitor.state = HealthState::Failing;
    } else if monitor.error_count > 0 {
        monitor.state = HealthState::Degraded;
    }

    // Se houve mudança para pior, notificar recuperação
    if monitor.state != old_state
        && (monitor.state == HealthState::Failing || monitor.state == HealthState::Dead)
    {
        crate::kerror!(
            "(Monitor) Saúde do dispositivo degradada criticamente, ID:",
            id.0
        );

        // Aciona o sistema de recuperação global
        super::report_failure(id, err);

        // Emite evento para o sistema
        super::events::emit(super::events::DeviceEvent::ErrorDetected(id, err));
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

// =============================================================================
// ROADMAP DE IMPLEMENTAÇÕES FUTURAS (PLANEJAMENTO)
// =============================================================================
//
// 1. Monitoramento de Performance (Throughput/Latency):
//    - Rastrear a velocidade de E/S. Se um SSD que deveria entregar 500MB/s
//      estiver entregando 1MB/s, marcar como Degraded (Degradação de Performance).
//
// 2. Análise Preditiva de Falhas (S.M.A.R.T Integrado):
//    - Estudar padrões de erros para prever falhas antes que elas ocorram.
//
// 3. Watchdog por Driver:
//    - Exigir que drivers enviem um "ping" periódico. Se o driver do mouse
//      travar em um loop infinito, o Monitor detecta o silêncio e reinicia o driver.
//
// 4. Interface Gráfica de Saúde:
//    - Exportar esses dados para uma ferramenta tipo "Gerenciador de Dispositivos"
//      no RedstoneOS GUI.
//
// 5. Histórico de Pânico de Hardware:
//    - Salvar o log de erros de saúde em uma partição de crash-dump para análise
//      após o reboot.
