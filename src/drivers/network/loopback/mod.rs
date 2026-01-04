//! # Loopback Interface
//!
//! Interface virtual para comunicação interna (localhost).

use super::traits::{LinkStatus, MacAddress, NetworkAdapter};
use crate::drivers::base::device::{Device, DeviceState};
use crate::drivers::base::driver::{DeviceType, Driver, DriverError};
use alloc::sync::Arc;
use alloc::vec::Vec;

pub struct LoopbackDriver;

impl Driver for LoopbackDriver {
    fn name(&self) -> &'static str {
        "Software Loopback Adapter"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Network
    }

    fn probe(&self, _dev: &mut Device) -> Result<(), DriverError> {
        crate::kinfo!("(Net/Loopback) Interface localhost ativa.");
        Ok(())
    }

    fn remove(&self, dev: &mut Device) -> Result<(), DriverError> {
        dev.state = DeviceState::Disconnected;
        Ok(())
    }
}

impl NetworkAdapter for LoopbackDriver {
    fn name(&self) -> &'static str {
        "lo"
    }

    fn mac_address(&self) -> MacAddress {
        MacAddress::ZERO // Loopback não tem MAC físico
    }

    fn link_status(&self) -> LinkStatus {
        LinkStatus::Up
    }

    fn transmit(&self, _packet: &[u8]) -> Result<(), &'static str> {
        // STUB: Loopback imediato para a fila de recepção do kernel
        // 1. Receber pacote
        // 2. Colocar em um buffer interno ou despachar direto para o stack
        Ok(())
    }

    fn receive(&self) -> Option<Vec<u8>> {
        None
    }
}
