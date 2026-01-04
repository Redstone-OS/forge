//! # System Drivers Subsystem
//!
//! Drivers for core system components like timers, interrupt controllers,
//! DMA engines, and power management.

pub mod dma;
pub mod int_ctrl;
pub mod pwr_ctl;
pub mod speaker;
pub mod timer;

/// Initialize all system drivers.
pub fn init() {
    crate::kinfo!("(System) Initializing system hardware...");

    // 1. Interrupt Controllers (PIC/APIC) - Critical for handling IRQs
    int_ctrl::init();

    // 2. Timers (PIT/HPET/TSC) - Critical for scheduling
    timer::init();

    // 3. DMA Controllers (8237)
    dma::init();

    // 4. Power Control
    pwr_ctl::init();

    // 5. Miscellaneous (PC Speaker)
    speaker::init();

    crate::kinfo!("(System) System hardware ready.");
}
