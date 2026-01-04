//! # Parallel Port Driver (IEEE 1284)
//!
//! Driver legado para suporte a portas paralelas (LPT).
//! Utilizado hoje em dia principalmente para depuração e hardware industrial antigo.

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct ParallelDriver;

impl Driver for ParallelDriver {
    fn name(&self) -> &'static str {
        "Standard Parallel Port Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Generic
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kdebug!("(Parallel) Porta LPT detectada.");
        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(ParallelDriver));
}
