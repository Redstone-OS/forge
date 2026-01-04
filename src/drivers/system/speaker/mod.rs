//! # PC Speaker Driver
//!
//! Driver simples para controlar o alto-falante do sistema (beep).

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct SpeakerDriver;

impl Driver for SpeakerDriver {
    fn name(&self) -> &'static str {
        "PC Speaker Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::System
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(System/Speaker) PC Speaker registrado.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

/// Toca um som na frequência especificada
pub fn play(frequency: u32) {
    // STUB: Configurar PIT Channel 2
    // 1. Calcular divisor (1193180 / frequency)
    // 2. Enviar comando para PIT_COMMAND (0xB6)
    // 3. Enviar divisor para PIT_CHANNEL2 (0x42)
    // 4. Habilitar bits 0 e 1 da porta 0x61
}

/// Para o som
pub fn stop() {
    // STUB: Desabilitar bits 0 e 1 da porta 0x61
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(SpeakerDriver) as Arc<dyn Driver>);
}
