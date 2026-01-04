//! # Interrupt Controller Driver Manager
//!
//! Orquestra os controladores de interrupção (PIC, APIC, IO-APIC).

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct IntCtrlDriver;

impl Driver for IntCtrlDriver {
    fn name(&self) -> &'static str {
        "Interrupt Controller Manager"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::System
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Detecção de topologia de interrupção
        // 1. Checar CPUID para suporte a APIC
        // 2. Parsear tabelas MADT/ACPI
        // 3. Inicializar Local APIC
        // 4. Inicializar IO-APIC
        crate::kinfo!("(System/IntCtrl) Controladores de interrupção detectados.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(IntCtrlDriver) as Arc<dyn Driver>);
}
