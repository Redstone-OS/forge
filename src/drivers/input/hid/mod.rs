//! # Human Interface Device (HID) Subsystem
//!
//! Este módulo gerencia dispositivos de interface humana, fornecendo
//! parsing universal de relatórios HID. Ele não lida com o transporte físico
//! (USB, BT, I2C), mas sim com a semântica dos dados.
//!
//! ## Arquitetura:
//! - **usage**: Definições de Usage Pages e Usages (Keyboard, Mouse, etc).
//! - **report**: Parser de descritores de relatório.
//! - **types**: Coleções e campos internos.

pub mod report;
pub mod types;
pub mod usage;

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct HidSubsystem;

impl HidSubsystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl Driver for HidSubsystem {
    fn name(&self) -> &'static str {
        "Universal HID Subsystem"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Input
    }

    fn probe(&self, dev: &mut Device) -> Result<(), DriverError> {
        // O subsistema HID é um driver de "classe". Ele é chamado quando
        // um dispositivo de barramento (USB/I2C/Bluetooth) é identificado como HID.

        crate::kinfo!("(Input/HID) Dispositivo HID detectado. Iniciando enumeração lógica.");

        // 1. O barramento deve fornecer o Report Descriptor via dados privados do dispositivo.
        // 2. Criamos um HidReportParser para entender a estrutura dos dados.
        // 3. Mapeamos os bits para eventos de Input do RedstoneOS.

        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

pub fn init() {
    crate::drivers::base::register_driver(Arc::new(HidSubsystem::new()));
}
