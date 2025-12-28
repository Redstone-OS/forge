/// Arquivo: core/power/state.rs
///
/// Propósito: Definição dos Estados de Energia do Sistema.
/// Baseado na especificação ACPI (G-States e S-States).
///
/// Detalhes de Implementação:
/// - G0/S0: Working (Ligado)
/// - G1: Sleeping (S1-S4)
/// - G2/S5: Soft Off
/// - G3: Mechanical Off

//! Estados de Energia (System Power States)

use core::sync::atomic::{AtomicU8, Ordering};

/// Estados Globais de Energia (ACPI)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PowerState {
    /// G0 (S0): Working - Sistema totalmente operacional.
    Working = 0,
    
    /// G1 (S1): Sleeping - CPU parada, mas caches mantidos (baixa latência).
    Standby = 1,
    
    /// G1 (S3): Suspend-to-RAM - Contexto salvo na RAM, a maioria dos devices desligados.
    SuspendToRam = 3,
    
    /// G1 (S4): Hibernation - Contexto salvo em disco, sistema desligado.
    Hibernate = 4, // "Suspend-to-Disk"
    
    /// G2 (S5): Soft Off - Sistema desligado via software, mas fonte energizada.
    SoftOff = 5,
    
    /// G3: Mechanical Off - Sem energia nenhuma (apenas RTC bateria).
    MechanicalOff = 6,
}

impl From<u8> for PowerState {
    fn from(val: u8) -> Self {
        match val {
            0 => PowerState::Working,
            1 => PowerState::Standby,
            3 => PowerState::SuspendToRam,
            4 => PowerState::Hibernate,
            5 => PowerState::SoftOff,
            6 => PowerState::MechanicalOff,
            _ => PowerState::Working, // Default seguro
        }
    }
}

// Estado atual do sistema.
static CURRENT_STATE: AtomicU8 = AtomicU8::new(PowerState::Working as u8);

/// Retorna o estado atual de energia.
pub fn current_state() -> PowerState {
    PowerState::from(CURRENT_STATE.load(Ordering::Relaxed))
}

/// Define o estado atual (uso interno pelo gerenciador de power).
pub(crate) fn set_state(state: PowerState) {
    CURRENT_STATE.store(state as u8, Ordering::Relaxed);
}
