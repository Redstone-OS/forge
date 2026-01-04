//! # xHCI Device Management
//!
//! Alocação de slots e gerenciamento de contextos de dispositivos.

use super::structs::*;
use super::types::*;

/// Aloca um slot para um novo dispositivo.
///
/// ## STUB:
/// Não envia comando real.
pub fn allocate_slot() -> Option<u8> {
    crate::kwarn!("(xHCI Device) allocate_slot() stub");

    // TODO:
    // 1. Enviar Enable Slot Command
    // 2. Aguardar completion no Event Ring
    // 3. Retornar slot ID

    None
}

/// Libera um slot.
pub fn free_slot(_slot_id: u8) {
    crate::kwarn!("(xHCI Device) free_slot() stub");

    // TODO: Enviar Disable Slot Command
}

/// Atribui endereço a um dispositivo.
///
/// ## STUB:
/// Não envia comando real.
pub fn address_device(_slot_id: u8, _port: u8, _speed: u8) -> Option<u8> {
    crate::kwarn!("(xHCI Device) address_device() stub");

    // TODO:
    // 1. Preparar Input Context
    // 2. Enviar Address Device Command
    // 3. Aguardar completion
    // 4. Retornar device address

    None
}

/// Configura endpoints de um dispositivo.
pub fn configure_endpoints(_slot_id: u8, _endpoints: &[EndpointConfig]) -> bool {
    crate::kwarn!("(xHCI Device) configure_endpoints() stub");

    // TODO: Enviar Configure Endpoint Command

    false
}

/// Configuração de endpoint.
#[derive(Debug, Clone)]
pub struct EndpointConfig {
    pub endpoint_num: u8,
    pub direction_in: bool,
    pub transfer_type: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}
