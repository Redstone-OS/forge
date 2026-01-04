//! # USB Mass Storage Driver
//!
//! Este módulo implementa o driver de **USB Mass Storage** (classe 0x08),
//! permitindo acesso a pen drives, HDs externos, etc.
//!
//! ## Protocolo:
//! - **BBB**: Bulk-Only Transport (mais comum)
//! - **CBI**: Control/Bulk/Interrupt (obsoleto)
//!
//! ## SCSI Commands:
//! USB Mass Storage usa comandos SCSI encapsulados em CBWs.
//! Comandos principais: INQUIRY, READ(10), WRITE(10), TEST UNIT READY
//!
//! ## Arquitetura:
//! ```text
//! VFS/Block Layer
//!        ↓
//! USB Mass Storage Driver (este módulo)
//!        ↓
//! SCSI Command Layer (scsi.rs)
//!        ↓
//! USB Bulk Transport (transport.rs)
//!        ↓
//! xHCI/EHCI
//! ```
//!
//! ## STUB:
//! Estruturas e constantes definidas. Implementação incompleta.

pub mod scsi; // Comandos SCSI
pub mod transport; // BBB transport

use crate::drivers::bus::usb::device::UsbDevice;
// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::drivers::bus::usb::types::*;
use crate::sync::Spinlock;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Classe USB para Mass Storage.
pub const USB_CLASS_MASS_STORAGE: u8 = 0x08;

/// Subclasse SCSI transparent command set.
pub const USB_SUBCLASS_SCSI: u8 = 0x06;

/// Protocolo Bulk-Only (BBB).
pub const USB_PROTOCOL_BBB: u8 = 0x50;

// =============================================================================
// ESTRUTURA DE DISPOSITIVO
// =============================================================================

/// Representa um dispositivo USB Mass Storage.
#[derive(Debug, Clone)]
pub struct MassStorageDevice {
    /// Endereço USB do dispositivo.
    pub usb_address: u8,

    /// Endpoint Bulk IN.
    pub bulk_in_endpoint: u8,

    /// Endpoint Bulk OUT.
    pub bulk_out_endpoint: u8,

    /// Max packet size dos endpoints bulk.
    pub max_packet_size: u16,

    /// Max LUN (Logical Unit Number).
    pub max_lun: u8,

    /// Capacidade em blocos.
    pub block_count: u64,

    /// Tamanho do bloco em bytes.
    pub block_size: u32,

    /// Vendor ID do dispositivo.
    pub vendor_id: u16,

    /// Product ID.
    pub product_id: u16,

    /// String do produto.
    pub product_name: Option<alloc::string::String>,

    /// Dispositivo está pronto?
    pub ready: bool,
}

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Lista de dispositivos mass storage detectados.
static DEVICES: Spinlock<Vec<MassStorageDevice>> = Spinlock::new(Vec::new());

// =============================================================================
// FUNÇÕES PÚBLICAS
// =============================================================================

/// Inicializa um dispositivo USB como Mass Storage.
///
/// ## STUB:
/// Não inicializa realmente.
pub fn probe(usb_dev: &UsbDevice) -> bool {
    crate::kinfo!(
        "(USB MSC) Probing device:",
        usb_dev.vendor_id,
        ":",
        usb_dev.product_id
    );

    // Verifica se é mass storage
    if !usb_dev.is_mass_storage() {
        return false;
    }

    crate::kinfo!("(USB MSC) Dispositivo é Mass Storage!");

    // TODO: Implementar
    // 1. Buscar endpoints Bulk IN e OUT
    // 2. Enviar GET_MAX_LUN
    // 3. Enviar INQUIRY
    // 4. Enviar READ_CAPACITY
    // 5. Registrar como block device

    crate::kwarn!("(USB MSC) probe() não totalmente implementado");

    false
}

/// Lê blocos de um dispositivo.
///
/// ## STUB:
/// Não lê realmente.
// TODO: Revisar no futuro
#[allow(unused_variables)]
pub fn read_blocks(
    _device_index: usize,
    _start_block: u64,
    _count: u32,
    _buffer: &mut [u8],
) -> Result<(), MscError> {
    crate::kwarn!("(USB MSC) read_blocks() stub");

    // TODO: Implementar via SCSI READ(10)

    Err(MscError::NotImplemented)
}

/// Escreve blocos em um dispositivo.
///
/// ## STUB:
/// Não escreve realmente.
// TODO: Revisar no futuro
#[allow(unused_variables)]
pub fn write_blocks(
    _device_index: usize,
    _start_block: u64,
    _count: u32,
    _buffer: &[u8],
) -> Result<(), MscError> {
    crate::kwarn!("(USB MSC) write_blocks() stub");

    // TODO: Implementar via SCSI WRITE(10)

    Err(MscError::NotImplemented)
}

/// Retorna número de dispositivos mass storage.
pub fn device_count() -> usize {
    DEVICES.lock().len()
}

/// Retorna informações de um dispositivo.
pub fn get_device_info(index: usize) -> Option<MassStorageDevice> {
    DEVICES.lock().get(index).cloned()
}

// =============================================================================
// ERROS
// =============================================================================

/// Erros de Mass Storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MscError {
    /// Dispositivo não encontrado.
    NoDevice,
    /// Erro de USB.
    UsbError,
    /// Erro de SCSI.
    ScsiError,
    /// Timeout.
    Timeout,
    /// Buffer inválido.
    InvalidBuffer,
    /// Não implementado.
    NotImplemented,
}
