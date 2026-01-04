//! # I2C Bus Driver (Inter-Integrated Circuit)
//!
//! Suporte a barramento I2C para comunicação com sensores,
//! relógios de tempo real (RTC) externos e EEPROMs.

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct I2cDriver;

impl Driver for I2cDriver {
    fn name(&self) -> &'static str {
        "Generic I2C/SMBus Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Bus
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kdebug!("(I2C) Probing barramento I2C...");
        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(I2cDriver));
}
