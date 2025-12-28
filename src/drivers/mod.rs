//! # Drivers
//!
//! Drivers de hardware e modelo de dispositivos.

pub mod base;
pub mod block;
pub mod input;
pub mod irq;
pub mod net;
pub mod pci;
pub mod serial;
pub mod timer;
pub mod video; // Legacy

pub use base::{Device, DeviceType, Driver, DriverError};

/// Inicializa sistema de drivers
pub fn init() {
    crate::kinfo!("(Drivers) Inicializando sistema de drivers...");

    // 1. Inicializar drivers base
    base::init();

    // 2. Inicializar Serial (já deve ter sido init no boot, mas aqui registra driver)
    serial::init();

    // 3. Inicializar Timers
    timer::init_pit(1000); // 1000 Hz

    // 4. Inicializar Vídeo
    // (Video é inicializado no kernel_main com BootInfo)

    // 5. Detectar PCI
    pci::scan();

    crate::kinfo!("(Drivers) Drivers inicializados.");
}
