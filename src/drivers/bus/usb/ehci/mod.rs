//! # Driver EHCI (USB 2.0)
//!
//! Controladores Legados USB 2.0. (Placeholder)

use super::super::base::device::{Device, DeviceState};
use super::super::base::driver::{DeviceType, Driver, DriverError};
use super::host::{UsbDeviceDescriptor, UsbError, UsbHostController, UsbPortEvent};
use crate::sync::Spinlock;
use alloc::sync::Arc;

pub struct EhciDriver;

impl Driver for EhciDriver {
    fn name(&self) -> &'static str {
        "EHCI USB 2.0 Host Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Controller
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // EHCI não implementado ainda, mas a infraestrutura está pronta.
        Err(DriverError::NotSupported)
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(EhciDriver));
}
