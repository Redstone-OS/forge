//! # VirtIO-Net Driver
//!
//! Implementação para placas de rede virtualizadas (Paravirtualização).

use super::traits::{LinkStatus, MacAddress, NetworkAdapter};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;

pub struct VirtioNetDriver;

impl Driver for VirtioNetDriver {
    fn name(&self) -> &'static str {
        "VirtIO Networking Driver"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Network
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        // STUB: Lógica VirtIO-Net
        // 1. Handshake do protocolo VirtIO
        // 2. Negociar Features (MAC, Status, MRG_RXBUF)
        // 3. Configurar Virtqueues (0: Receive, 1: Transmit)
        // 4. Preencher a Receive Queue com buffers vazios
        // 5. Mudar estado para DRIVER_OK

        crate::kinfo!("(Net/Virtio) Inicializando interface VirtIO-Net.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

impl NetworkAdapter for VirtioNetDriver {
    fn name(&self) -> &'static str {
        "virtio-net"
    }

    fn mac_address(&self) -> MacAddress {
        // TODO: Ler da struct de configuração do VirtIO
        MacAddress::ZERO
    }

    fn link_status(&self) -> LinkStatus {
        LinkStatus::Up // Virtio geralmente está sempre UP se houver backend
    }

    fn transmit(&self, _packet: &[u8]) -> Result<(), &'static str> {
        // TODO: Adicionar à Virtqueue 1 e notificar o host
        Err("Not implemented")
    }

    fn receive(&self) -> Option<alloc::vec::Vec<u8>> {
        // TODO: Checar se o host colocou dados na Virtqueue 0
        None
    }
}
