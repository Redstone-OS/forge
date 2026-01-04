//! # USB Device Representation
//!
//! Objetos que representam dispositivos USB individuais no Redstone Driver Model.

use super::host::UsbHostController;
use super::types::{UsbDeviceDescriptor, UsbSpeed};
use alloc::sync::Arc;

/// Dados privados de um dispositivo USB anexados ao `Device::data`.
pub struct UsbDeviceData {
    /// O controlador host ao qual este dispositivo está conectado
    pub host: Arc<dyn UsbHostController>,
    /// Endereço do hub (0 se conectado diretamente ao Root Hub)
    pub hub_addr: u8,
    /// Porta no hub/controlador
    pub port: u8,
    /// Velocidade de negociação
    pub speed: UsbSpeed,
    /// Descritor de dispositivo (Identidade)
    pub descriptor: UsbDeviceDescriptor,
}

impl UsbDeviceData {
    pub fn new(
        host: Arc<dyn UsbHostController>,
        port: u8,
        speed: UsbSpeed,
        descriptor: UsbDeviceDescriptor,
    ) -> Self {
        Self {
            host,
            hub_addr: 0, // Por enquanto apenas Root Hubs
            port,
            speed,
            descriptor,
        }
    }
}
