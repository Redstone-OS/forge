//! # Power Control Driver
//!
//! Gerencia estados de energia do sistema (Shutdown, Reboot, ACPI Sleep).

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct PowerControlDriver;

impl Driver for PowerControlDriver {
    fn name(&self) -> &'static str {
        "System Power Controller"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::System
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(System/Power) Controle de energia ativo.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

/// Reinicia o sistema
pub fn reboot() {
    // STUB: Métodos de reset
    // 1. ACPI Reset (FADT)
    // 2. Teclado 8042 (0x64 <- 0xFE)
    // 3. Triple Fault
    crate::kwarn!("(System) Reiniciando...");
}

/// Desliga o sistema
pub fn shutdown() {
    // STUB: Métodos de shutdown
    // 1. ACPI S5 (Sleep Type A/B nos registradores PM1a/PM1b)
    // 2. QEMU Debug Exit (se em VM)
    crate::kwarn!("(System) Desligando...");
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(PowerControlDriver) as Arc<dyn Driver>);
}
