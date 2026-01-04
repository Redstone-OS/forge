//! # VirtIO Input Driver
//!
//! Driver para dispositivos de entrada virtualizados (QEMU virtio-input-pci).

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct VirtioInputDriver;

impl Driver for VirtioInputDriver {
    fn name(&self) -> &'static str {
        "VirtIO Input Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Input
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(VirtioInputDriver));
}
