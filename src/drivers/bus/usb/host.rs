//! # USB Host Controller Interface
//!
//! Define a abstração para controladores host (xHCI, EHCI, etc).

use super::types::{UsbDeviceDescriptor, UsbSpeed};
use alloc::vec::Vec;

/// Interface que todo controlador USB deve implementar para ser gerenciado pelo UsbBus
pub trait UsbHostController: Send + Sync {
    /// Nome do controlador (ex: "xHCI Controller 0")
    fn name(&self) -> &'static str;

    /// Escaneia as portas do controlador em busca de mudanças de estado
    fn poll_ports(&self) -> Vec<UsbPortEvent>;

    /// Realiza o ciclo de vida inicial de um dispositivo detectado
    /// Retorna o DeviceDescriptor se o endereçamento e handshake inicial funcionarem.
    fn setup_device(&self, port_id: u8) -> Result<UsbDeviceDescriptor, UsbError>;

    /// Envia uma transferência USB (Control, Bulk, Interrupt, Iso)
    fn transfer(&self, request: UsbTransferRequest) -> Result<(), UsbError>;
}

#[derive(Debug)]
pub enum UsbError {
    Timeout,
    Babble,
    Stall,
    HardwareError,
    NoMemory,
}

#[derive(Debug)]
pub struct UsbPortEvent {
    pub port_id: u8,
    pub connected: bool,
    pub speed: UsbSpeed,
}

pub struct UsbTransferRequest {
    // TODO: Definir campos para transferências universais
}
