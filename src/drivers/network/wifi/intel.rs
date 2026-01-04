//! # Intel Wireless Driver (iwlwifi style)
//!
//! Suporte para chipsets modernos da Intel (Wireless-AC, AX).

use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};

pub struct IntelWifiDriver;

impl Driver for IntelWifiDriver {
    fn name(&self) -> &'static str {
        "Intel Wireless WiFi Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Network
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Inicialização iwlwifi
        // 1. Mapear MMIO BAR
        // 2. Carregar Firmware binário (.ucode)
        // 3. Configurar DMA e Command Queue
        // 4. Receber ALIVE notification do microcode

        crate::kinfo!("(Net/Wifi) Adaptador Intel Wireless detectado (stub).");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}
