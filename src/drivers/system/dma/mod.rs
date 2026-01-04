//! # DMA Controller Driver (legacy 8237)
//!
//! Gerencia o controlador DMA legado usado por Floppy e Sound Blaster.

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct DmaDriver;

impl Driver for DmaDriver {
    fn name(&self) -> &'static str {
        "Legacy DMA Controller (8237)"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::System
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Inicialização DMA
        // 1. Resetar controlador (Master/Slave)
        // 2. Mascarar todos os canais
        crate::kinfo!("(System/DMA) Controlador DMA legado inicializado (stub).");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(DmaDriver) as Arc<dyn Driver>);
}
