//! # I2C-HID Touchpad Driver
//!
//! Suporte a touchpads Synaptics e ELAN comuns em notebooks.
//! Gerencia a detecção, inicialização e processamento de eventos via RDM.

pub mod hid;
pub mod i2c;
pub mod protocol;
pub mod state;
pub mod utils;

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use crate::sync::Spinlock;
use alloc::sync::Arc;

use hid::{commands as hid_cmd, power, TouchpadState, TouchpadType};
use i2c::I2cController;
use state::*;

pub struct TouchpadDriver;

impl TouchpadDriver {
    pub fn new() -> Self {
        Self {}
    }
}

impl Driver for TouchpadDriver {
    fn name(&self) -> &'static str {
        "I2C-HID Touchpad Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Input
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(Input/Touch) Iniciando probe de touchpad I2C-HID.");

        // TODO: Em um sistema RDM real, o barramento (I2C/Platform)
        // já teria passado o endereço MMIO e IRQ.
        // Por enquanto, usamos a lógica de detecção estática nos utilitários.

        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

/// Handler de interrupção do touchpad (chamado pelo sistema de interrupções)
pub fn handle_irq() {
    let mut i2c_guard = I2C.lock();
    let i2c = match i2c_guard.as_mut() {
        Some(c) => c,
        None => return,
    };

    let desc_guard = HID_DESC.lock();
    let desc = match desc_guard.as_ref() {
        Some(d) => d,
        None => return,
    };

    // Ler input report
    let input_reg = desc.input_register.to_le_bytes();
    let max_len = desc.max_input_length.min(64) as usize;
    drop(desc_guard);

    let mut buf = [0u8; 64];
    if i2c.write_read(&input_reg, &mut buf[..max_len]).is_err() {
        return;
    }

    let report_len = u16::from_le_bytes([buf[0], buf[1]]) as usize;
    if report_len < 4 || report_len > max_len {
        return;
    }

    // Despachar para o protocolo correto
    let tp_type = *TOUCHPAD_TYPE.lock();
    protocol::process_report(tp_type, &buf[2..report_len]);
}

/// Obtém o estado atual do touchpad
pub fn get_state() -> TouchpadState {
    let mut state = TOUCHPAD_STATE.lock();
    let current = *state;
    state.delta_x = 0;
    state.delta_y = 0;
    current
}

/// Registra o driver no subsistema
pub fn init() {
    crate::drivers::base::register_driver(Arc::new(TouchpadDriver::new()) as Arc<dyn Driver>);
}

// Re-exports de compatibilidade
pub fn set_resolution(width: i32, height: i32) {
    let mut state = TOUCHPAD_STATE.lock();
    state.screen_width = width;
    state.screen_height = height;
    state.cursor_x = width / 2;
    state.cursor_y = height / 2;
}

pub fn is_initialized() -> bool {
    I2C.lock().is_some()
}
