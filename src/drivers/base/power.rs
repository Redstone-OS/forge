//! # Gerenciamento de Energia (Power Management)
//!
//! Este módulo coordena o **consumo elétrico** dos dispositivos de hardware.
//! Essencial para:
//! - Economia de bateria em notebooks
//! - Redução de consumo em servidores
//! - Gerenciamento térmico
//! - Suspensão/hibernação do sistema
//!
//! ## D-States (Device Power States):
//! Baseados na especificação ACPI/PCI:
//! - **D0**: Full Power - Totalmente operacional
//! - **D1**: Light Sleep - Baixa latência para acordar
//! - **D2**: Deep Sleep - Maior economia, maior latência
//! - **D3**: Off - Desligado, estado pode ser perdido
//!
//! ## Regras de Transição:
//! - Dispositivos filhos devem ser suspensos ANTES dos pais
//! - Dispositivos pais devem ser acordados ANTES dos filhos
//! - Barramento deve estar ativo para acordar dispositivos

use super::device::DeviceId;
use crate::sync::Spinlock;
use alloc::vec::Vec;

// =============================================================================
// ESTADOS DE ENERGIA
// =============================================================================

/// Estados de energia padrão (baseados em ACPI/PCI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PowerState {
    /// Full Power: Dispositivo totalmente operacional.
    /// Máximo consumo, mínima latência.
    D0,

    /// Light Sleep: Economia mínima, acordar rápido.
    /// Clock pode estar reduzido.
    D1,

    /// Deep Sleep: Maior economia, acordar lento.
    /// Alguns registradores podem ser perdidos.
    D2,

    /// Off: Desligado completamente.
    /// Todo estado interno pode ser perdido.
    /// Precisa de re-inicialização ao acordar.
    D3,
}

impl PowerState {
    /// Retorna nome legível do estado.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::D0 => "D0 (Full Power)",
            Self::D1 => "D1 (Light Sleep)",
            Self::D2 => "D2 (Deep Sleep)",
            Self::D3 => "D3 (Off)",
        }
    }

    /// Retorna consumo relativo (0-100).
    pub fn power_consumption(&self) -> u8 {
        match self {
            Self::D0 => 100,
            Self::D1 => 50,
            Self::D2 => 10,
            Self::D3 => 0,
        }
    }

    /// Retorna latência aproximada para acordar (em ms).
    pub fn wake_latency_ms(&self) -> u32 {
        match self {
            Self::D0 => 0,
            Self::D1 => 1,
            Self::D2 => 10,
            Self::D3 => 100,
        }
    }
}

// =============================================================================
// EVENTOS DE ENERGIA
// =============================================================================

/// Eventos de transição de energia.
#[derive(Debug, Clone, Copy)]
pub enum PowerEvent {
    /// Preparando para suspender (salvar estado).
    PrepareSuspend,

    /// Suspensão em andamento.
    Suspend,

    /// Preparando para acordar (restaurar estado).
    PrepareResume,

    /// Acordando.
    Resume,

    /// Preparando para desligar.
    PrepareShutdown,

    /// Desligando.
    Shutdown,
}

// =============================================================================
// TRAIT DE GERENCIAMENTO DE ENERGIA
// =============================================================================

/// Interface que drivers devem implementar para suportar PM.
pub trait PowerManaged: Send + Sync {
    /// Altera o estado de energia do hardware.
    ///
    /// ## Retorno:
    /// - true: Transição bem-sucedida
    /// - false: Transição não suportada ou falhou
    fn set_power_state(&self, state: PowerState) -> bool;

    /// Retorna o estado atual de energia.
    fn get_power_state(&self) -> PowerState;

    /// Salva contexto antes de suspensão.
    ///
    /// Chamado quando o sistema vai entrar em suspensão.
    /// Driver deve salvar registradores que serão perdidos.
    fn save_context(&self) -> bool {
        // Default: sucesso (nada a salvar)
        true
    }

    /// Restaura contexto após acordar.
    ///
    /// Chamado quando o sistema acorda.
    /// Driver deve restaurar registradores salvos.
    fn restore_context(&self) -> bool {
        // Default: sucesso (nada a restaurar)
        true
    }

    /// Retorna latência máxima aceitável para acordar (em ms).
    ///
    /// Usado pelo PowerManager para decidir até qual D-state ir.
    fn max_wake_latency_ms(&self) -> u32 {
        100 // Default: aceita até 100ms
    }
}

// =============================================================================
// REGISTRO DE ESTADO DE ENERGIA
// =============================================================================

/// Registro de estado de energia de um dispositivo.
#[derive(Debug, Clone)]
struct PowerRecord {
    device_id: DeviceId,
    current_state: PowerState,
    supports_d1: bool,
    supports_d2: bool,
    supports_d3: bool,
}

// =============================================================================
// GERENCIADOR DE ENERGIA
// =============================================================================

/// Gerenciador global de energia.
pub struct PowerManager {
    /// Estado de energia do sistema como um todo.
    system_state: PowerState,

    /// Registros de energia por dispositivo.
    records: Vec<PowerRecord>,

    /// Flag de inicialização.
    initialized: bool,
}

/// Instância global do PowerManager.
static POWER_MANAGER: Spinlock<PowerManager> = Spinlock::new(PowerManager {
    system_state: PowerState::D0,
    records: Vec::new(),
    initialized: false,
});

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa o subsistema de energia.
pub fn init() {
    crate::kinfo!("(Power) Inicializando Power Manager...");

    let mut mgr = POWER_MANAGER.lock();
    mgr.initialized = true;

    crate::kinfo!("(Power) Sistema pronto");
}

/// Solicita suspensão de um dispositivo específico.
///
/// ## STUB:
/// Implementação completa requer integração com DriverContext.
pub fn suspend_device(id: DeviceId) -> bool {
    crate::kinfo!("(Power) Solicitando suspensão para ID:", id.0);

    crate::kwarn!("(Power) suspend_device() não totalmente implementado");

    // TODO: Implementar
    // 1. Verificar se dispositivo suporta PM
    // 2. Salvar contexto
    // 3. Chamar driver.suspend()
    // 4. Atualizar registro

    true
}

/// Acorda um dispositivo específico.
///
/// ## STUB:
/// Implementação completa requer integração com DriverContext.
pub fn resume_device(id: DeviceId) -> bool {
    crate::kinfo!("(Power) Solicitando retomada para ID:", id.0);

    crate::kwarn!("(Power) resume_device() não totalmente implementado");

    // TODO: Implementar
    // 1. Verificar estado atual
    // 2. Chamar driver.resume()
    // 3. Restaurar contexto
    // 4. Atualizar registro

    true
}

/// Suspende todos os dispositivos (sistema entrando em sleep).
///
/// Respeita hierarquia: folhas primeiro, depois troncos.
pub fn suspend_all() -> bool {
    crate::kinfo!("(Power) Iniciando suspensão global...");

    let mut mgr = POWER_MANAGER.lock();

    // TODO: Percorrer árvore de dispositivos em ordem reversa
    // Por enquanto, apenas marca estado

    mgr.system_state = PowerState::D3;

    crate::kinfo!("(Power) Sistema suspenso");
    true
}

/// Acorda todos os dispositivos (sistema acordando).
///
/// Respeita hierarquia: troncos primeiro, depois folhas.
pub fn resume_all() {
    crate::kinfo!("(Power) Iniciando retomada global...");

    let mut mgr = POWER_MANAGER.lock();
    mgr.system_state = PowerState::D0;

    // TODO: Percorrer árvore de dispositivos em ordem de nível
    // Barramentos primeiro, depois dispositivos

    crate::kinfo!("(Power) Sistema acordado");
}

/// Retorna o estado de energia global do sistema.
pub fn get_system_power_state() -> PowerState {
    POWER_MANAGER.lock().system_state
}

/// Retorna o estado de energia de um dispositivo.
pub fn get_device_power_state(id: DeviceId) -> Option<PowerState> {
    POWER_MANAGER
        .lock()
        .records
        .iter()
        .find(|r| r.device_id == id)
        .map(|r| r.current_state)
}

/// Registra capacidades de energia de um dispositivo.
pub fn register_device_power(
    id: DeviceId,
    supports_d1: bool,
    supports_d2: bool,
    supports_d3: bool,
) {
    let mut mgr = POWER_MANAGER.lock();

    mgr.records.push(PowerRecord {
        device_id: id,
        current_state: PowerState::D0,
        supports_d1,
        supports_d2,
        supports_d3,
    });
}
