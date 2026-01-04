//! # Realtek Ethernet Driver (RTL8139/RTL8169)
//!
//! Driver para placas de rede Realtek Fast/Gigabit Ethernet.

use super::super::traits::{LinkStatus, MacAddress, NetworkAdapter};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct RealtekNetDriver;

impl Driver for RealtekNetDriver {
    fn name(&self) -> &'static str {
        "Realtek Ethernet Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Network
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Lógica de inicialização Realtek
        // 1. Ligar a placa (Configuração de energia)
        // 2. Reset via software (Command Register)
        // 3. Configurar buffer de recepção (RBSTART)
        // 4. Configurar filtros de interrupção (IMR)
        // 5. Ativar transmissor e receptor (CR)

        crate::kinfo!("(Net/Realtek) Inicializando adaptador Realtek em modo stub.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

impl NetworkAdapter for RealtekNetDriver {
    fn name(&self) -> &'static str {
        "rtl8139"
    }

    fn mac_address(&self) -> MacAddress {
        // TODO: Ler registradores IDR0-IDR5
        MacAddress::ZERO
    }

    fn link_status(&self) -> LinkStatus {
        LinkStatus::Unknown
    }

    fn transmit(&self, _packet: &[u8]) -> Result<(), &'static str> {
        // TODO: Copiar para porta de Transmit 0-3 e disparar via TSAD
        Err("Not implemented")
    }

    fn receive(&self) -> Option<alloc::vec::Vec<u8>> {
        // TODO: Ler do buffer circular e atualizar o CAPR
        None
    }
}
