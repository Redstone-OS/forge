//! Tempo e Timers

pub mod clock;
pub mod hrtimer;
pub mod jiffies;
pub mod timer;

/// Inicializa subsistema de tempo
pub fn init() {
    crate::kinfo!("(Time) Init");
    // TODO: Init PIT if needed, or HPET/TSC via drivers
}
