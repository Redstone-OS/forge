//! # VirtIO Console Driver
//!
//! Implementação de comunicação serial virtual simplificada para ambientes
//! virtualizados (QEMU, Cloud).

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct VirtioConsoleDriver;

impl Driver for VirtioConsoleDriver {
    fn name(&self) -> &'static str {
        "VirtIO Console Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Serial
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // TODO: Implementar handshake VirtIO Console
        crate::kdebug!("(VirtIO-Console) Probing dispositivo...");
        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(VirtioConsoleDriver));
}
