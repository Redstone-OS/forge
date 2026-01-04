//! # USB Input Driver Bridge
//!
//! Conecta dispositivos de entrada USB ao Redstone Driver Model e ao subsistema HID.

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct UsbInputDriver;

impl Driver for UsbInputDriver {
    fn name(&self) -> &'static str {
        "USB Input Bridge Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Input
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // Mapear interrupções USB para eventos do HID subsystem
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(UsbInputDriver));
}
