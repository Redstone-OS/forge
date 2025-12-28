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
pub fn frequency() -> u64 {
    1000 // Assumindo 1000Hz do PIT por padrão
}

/// Delay bloqueante em milissegundos
pub fn delay_ms(ms: u64) {
    // Stub: loop simples ou usar PIT
    // Idealmente:
    let start = ticks();
    let freq = frequency();
    let target_ticks = (ms * freq) / 1000;
    while ticks() - start < target_ticks {
        core::hint::spin_loop();
    }
}
