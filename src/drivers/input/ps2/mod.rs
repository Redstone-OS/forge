//! # Drivers PS/2 (Teclado e Mouse)
//!
//! Implementação dos drivers para controlador PS/2 8042.
//! Integrado ao Redstone Driver Model (RDM).

pub mod io;
pub mod keyboard;
pub mod mouse;
pub mod ports;

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub use keyboard::{handle_irq as keyboard_irq, pop_scancode, read_scancode};
pub use mouse::{
    get_state as mouse_get_state, handle_irq as mouse_irq, read_packet, set_resolution,
    MousePacket, MouseState,
};

pub struct Ps2Driver;

impl Driver for Ps2Driver {
    fn name(&self) -> &'static str {
        "PS/2 legacy Controller Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Input
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(Input/PS2) Inicializando controlador 8042...");

        // Limpar buffer inicial
        io::flush_output();

        // Inicializar teclado
        keyboard::init();

        // Inicializar mouse
        mouse::init();

        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

/// Inicializa e registra o driver no RDM
pub fn init() {
    crate::drivers::base::register_driver(Arc::new(Ps2Driver) as Arc<dyn Driver>);
}

/// Inicializa apenas o teclado PS/2 (Legacy API)
pub fn init_keyboard() {
    io::flush_output();
    keyboard::init();
}

/// Inicializa apenas o mouse PS/2 (Legacy API)
pub fn init_mouse() {
    mouse::init();
}
