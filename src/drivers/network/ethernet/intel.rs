//! # Intel Ethernet Driver (e1000/e1000e)
//!
//! Driver para placas de rede Intel Gigabit Ethernet.

use super::super::traits::{LinkStatus, MacAddress, NetworkAdapter};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct IntelNetDriver;

impl IntelNetDriver {
    pub fn new() -> Self {
        Self {}
    }
}

impl Driver for IntelNetDriver {
    fn name(&self) -> &'static str {
        "Intel Gigabit Ethernet Driver (e1000)"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Network
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Lógica de inicialização Intel
        // 1. Mapear BAR0 (MMIO)
        // 2. Resetar hardware
        // 3. Ler MAC address dos registradores RAL/RAH ou EEPROM
        // 4. Configurar Transmit e Receive Descriptors (Ring Buffers)
        // 5. Habilitar interrupções e recepção

        crate::kinfo!("(Net/Intel) Inicializando adaptador Intel em modo stub.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

// STUB: Implementação da trait de hardware
impl NetworkAdapter for IntelNetDriver {
    fn name(&self) -> &'static str {
        "e1000"
    }

    fn mac_address(&self) -> MacAddress {
        // TODO: Retornar o MAC real lido do hardware
        MacAddress::ZERO
    }

    fn link_status(&self) -> LinkStatus {
        // TODO: Ler registrador de status (STATUS) bit 1 (LU - Link Up)
        LinkStatus::Unknown
    }

    fn transmit(&self, _packet: &[u8]) -> Result<(), &'static str> {
        // TODO: Colocar pacote no TX Ring e atualizar o Tail Pointer (TDT)
        Err("Not implemented")
    }

    fn receive(&self) -> Option<alloc::vec::Vec<u8>> {
        // TODO: Verificar se há pacotes no RX Ring (bit de DD - Descriptor Done)
        None
    }
}
