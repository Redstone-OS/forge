//! # SPI Bus Driver (Serial Peripheral Interface)
//!
//! Gerencia a comunicação síncrona com periféricos como memórias flash,
//! sensores e telas LCD simples.

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct SpiDriver;

impl Driver for SpiDriver {
    fn name(&self) -> &'static str {
        "Generic SPI Bus Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Bus
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kdebug!("(SPI) Probing controlador de barramento...");
        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(SpiDriver));
}
