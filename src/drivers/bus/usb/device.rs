//! # USB Device Structure
//!
//! Estrutura que representa um dispositivo USB conectado.

use super::types::*;
use alloc::string::String;
use alloc::vec::Vec;

// =============================================================================
// ESTRUTURA DE DISPOSITIVO
// =============================================================================

/// Representa um dispositivo USB conectado ao sistema.
#[derive(Debug, Clone)]
pub struct UsbDevice {
    /// Endereço atribuído ao dispositivo (1-127).
    pub address: u8,

    /// Slot no host controller (para xHCI).
    pub slot_id: u8,

    /// Porta do hub onde está conectado.
    pub port: u8,

    /// Endereço do hub pai (0 = root hub).
    pub parent_hub_address: u8,

    /// Velocidade de conexão.
    pub speed: UsbSpeed,

    /// Vendor ID.
    pub vendor_id: u16,

    /// Product ID.
    pub product_id: u16,

    /// Classe do dispositivo.
    pub device_class: u8,

    /// Subclasse.
    pub device_subclass: u8,

    /// Protocolo.
    pub device_protocol: u8,

    /// String do fabricante.
    pub manufacturer: Option<String>,

    /// String do produto.
    pub product: Option<String>,

    /// Número de série.
    pub serial_number: Option<String>,

    /// Configuração ativa.
    pub current_configuration: u8,

    /// Lista de interfaces.
    pub interfaces: Vec<UsbInterface>,

    /// Max packet size para endpoint 0.
    pub max_packet_size_0: u8,

    /// Estado do dispositivo.
    pub state: UsbDeviceState,
}

/// Estado de um dispositivo USB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDeviceState {
    /// Dispositivo detectado mas não enumerado.
    Attached,

    /// Endereço atribuído via SET_ADDRESS.
    Addressed,

    /// Configuração ativa via SET_CONFIGURATION.
    Configured,

    /// Suspenso.
    Suspended,

    /// Desconectado.
    Disconnected,
}

/// Interface USB dentro de uma configuração.
#[derive(Debug, Clone)]
pub struct UsbInterface {
    /// Número da interface.
    pub interface_number: u8,

    /// Alternate setting ativo.
    pub alternate_setting: u8,

    /// Classe da interface.
    pub interface_class: u8,

    /// Subclasse.
    pub interface_subclass: u8,

    /// Protocolo.
    pub interface_protocol: u8,

    /// Endpoints da interface.
    pub endpoints: Vec<UsbEndpoint>,
}

/// Endpoint USB.
#[derive(Debug, Clone)]
pub struct UsbEndpoint {
    /// Endereço do endpoint (número + direção).
    pub address: u8,

    /// Tipo de transferência.
    pub transfer_type: UsbTransferType,

    /// Max packet size.
    pub max_packet_size: u16,

    /// Intervalo de polling (para interrupt/isochronous).
    pub interval: u8,
}

impl UsbDevice {
    /// Cria um novo dispositivo vazio.
    pub fn new(port: u8, speed: UsbSpeed) -> Self {
        Self {
            address: 0,
            slot_id: 0,
            port,
            parent_hub_address: 0,
            speed,
            vendor_id: 0,
            product_id: 0,
            device_class: 0,
            device_subclass: 0,
            device_protocol: 0,
            manufacturer: None,
            product: None,
            serial_number: None,
            current_configuration: 0,
            interfaces: Vec::new(),
            max_packet_size_0: 8,
            state: UsbDeviceState::Attached,
        }
    }

    /// Verifica se é um hub.
    pub fn is_hub(&self) -> bool {
        self.device_class == 0x09
    }

    /// Verifica se é um dispositivo HID.
    pub fn is_hid(&self) -> bool {
        self.device_class == 0x03 || self.interfaces.iter().any(|i| i.interface_class == 0x03)
    }

    /// Verifica se é mass storage.
    pub fn is_mass_storage(&self) -> bool {
        self.device_class == 0x08 || self.interfaces.iter().any(|i| i.interface_class == 0x08)
    }

    /// Busca uma interface por classe.
    pub fn find_interface(&self, class: u8) -> Option<&UsbInterface> {
        self.interfaces.iter().find(|i| i.interface_class == class)
    }

    /// Busca um endpoint de um tipo específico.
    pub fn find_endpoint(
        &self,
        interface: u8,
        transfer_type: UsbTransferType,
        direction: UsbDirection,
    ) -> Option<&UsbEndpoint> {
        let iface = self
            .interfaces
            .iter()
            .find(|i| i.interface_number == interface)?;

        iface.endpoints.iter().find(|e| {
            e.transfer_type == transfer_type
                && ((e.address & 0x80 != 0) == (direction == UsbDirection::In))
        })
    }
}
