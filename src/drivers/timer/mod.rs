//! Timer Drivers

pub mod hpet;
pub mod pit;
pub mod tsc;

pub use pit::init as init_pit;

/// Retorna ticks atuais do sistema (monotônico)
pub fn ticks() -> u64 {
    // TODO: Implementar usando TSC ou HPET real
    crate::core::time::jiffies::get_jiffies()
}

/// Retorna frequência do timer base em Hz
///
/// NOTA: Deve corresponder ao valor passado para pit::init()
pub fn frequency() -> u64 {
    100 // Corresponde ao pit::init(100) definido no boot
        // TODO: Implementar usando TSC ou HPET real,
        // ler frequencia de variavle global e fazer o pit setar a vareavel
}

/// Delay em milissegundos com cooperative multitasking
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
        // Isso permite cooperative multitasking enquanto espera
        crate::sched::yield_now();
    }
}
