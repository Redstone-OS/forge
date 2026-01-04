//! # Drivers de Timer
//!
//! Gerencia os temporizadores do sistemas (PIT, HPET, TSC).

pub mod hpet;
pub mod pit;
pub mod tsc;

use crate::drivers::base::driver::Driver;
use alloc::sync::Arc;

pub use pit::init as init_pit;

/// Inicializa os drivers de timer no RDM
pub fn init() {
    // Registrar HPET no RDM (será ativado se ACPI detectar)
    crate::drivers::base::register_driver(Arc::new(hpet::HpetDriver) as Arc<dyn Driver>);
}

/// Retorna ticks atuais do sistema (monotônico)
pub fn ticks() -> u64 {
    // TODO: Implementar usando TSC ou HPET real quando disponíveis
    crate::core::time::jiffies::get_jiffies()
}

/// Retorna frequência do timer base em Hz
///
/// NOTA: Deve corresponder ao valor passado para pit::init()
pub fn frequency() -> u64 {
    100 // Corresponde ao pit::init(100) definido no boot
}

/// Delay em milissegundos com multitarefa cooperativa
///
/// Ao invés de fazer busy-wait, cede a CPU via yield_now() permitindo
/// que outros processos executem enquanto este espera.
pub fn delay_ms(ms: u64) {
    if ms == 0 {
        return;
    }

    let start = ticks();
    let freq = frequency();
    let target_ticks = (ms * freq) / 1000;

    while ticks() - start < target_ticks {
        // Ceder CPU para outros processos
        crate::sched::yield_now();
    }
}
