//! # xHCI Transfer Operations
//!
//! Operações de transferência USB via xHCI.

use super::structs::Trb;
use super::types::*;
use crate::drivers::bus::usb::host::UsbError;
use crate::drivers::bus::usb::types::*;

/// Executa uma Control Transfer.
///
/// ## STUB:
/// Não executa realmente.
pub fn control_transfer(
    _slot_id: u8,
    _setup: &UsbSetupPacket,
    _data: Option<&mut [u8]>,
) -> Result<usize, UsbError> {
    crate::kwarn!("(xHCI Transfer) control_transfer() stub");

    // TODO:
    // 1. Preparar Setup Stage TRB
    // 2. Se tem dados, preparar Data Stage TRB
    // 3. Preparar Status Stage TRB
    // 4. Enfileirar no Transfer Ring do endpoint 0
    // 5. Ring doorbell
    // 6. Aguardar completion no Event Ring

    Err(UsbError::Unknown)
}

/// Executa uma Bulk Transfer.
///
/// ## STUB:
/// Não executa realmente.
pub fn bulk_transfer(
    _slot_id: u8,
    _endpoint: u8,
    _data: &mut [u8],
    _direction: UsbDirection,
) -> Result<usize, UsbError> {
    crate::kwarn!("(xHCI Transfer) bulk_transfer() stub");

    // TODO:
    // 1. Preparar Normal TRBs (pode precisar de vários para dados grandes)
    // 2. Enfileirar no Transfer Ring do endpoint
    // 3. Ring doorbell
    // 4. Aguardar completion

    Err(UsbError::Unknown)
}

/// Executa uma Interrupt Transfer.
///
/// ## STUB:
/// Não executa realmente.
pub fn interrupt_transfer(
    _slot_id: u8,
    _endpoint: u8,
    _data: &mut [u8],
    _direction: UsbDirection,
) -> Result<usize, UsbError> {
    crate::kwarn!("(xHCI Transfer) interrupt_transfer() stub");

    Err(UsbError::Unknown)
}

/// Cria TRB de Setup Stage.
pub fn create_setup_trb(setup: &UsbSetupPacket, transfer_type: u8, cycle: bool) -> Trb {
    let mut trb = Trb::new();

    // Parameter: setup packet (8 bytes)
    trb.parameter = unsafe { core::mem::transmute_copy(setup) };

    // Status: TRT (Transfer Type) e length=8
    trb.status = 8 | ((transfer_type as u32) << 16);

    // Control: TRB type = Setup, IDT=1 (Immediate Data)
    trb.set_type(TRB_TYPE_SETUP);
    trb.control |= 1 << 6; // IDT
    trb.set_cycle(cycle);

    trb
}

/// Cria TRB de Data Stage.
pub fn create_data_trb(buffer_ptr: u64, length: u32, direction_in: bool, cycle: bool) -> Trb {
    let mut trb = Trb::new();

    trb.parameter = buffer_ptr;
    trb.status = length;

    trb.set_type(TRB_TYPE_DATA);
    if direction_in {
        trb.control |= 1 << 16; // DIR = 1 (IN)
    }
    trb.set_cycle(cycle);

    trb
}

/// Cria TRB de Status Stage.
pub fn create_status_trb(direction_in: bool, cycle: bool) -> Trb {
    let mut trb = Trb::new();

    trb.set_type(TRB_TYPE_STATUS);

    // Para control read, status é OUT (device → host)
    // Para control write, status é IN (host → device)
    if !direction_in {
        trb.control |= 1 << 16; // DIR = 1 (IN)
    }

    trb.control |= 1 << 5; // IOC (Interrupt on Completion)
    trb.set_cycle(cycle);

    trb
}

/// Cria TRB Normal (para Bulk/Interrupt).
pub fn create_normal_trb(buffer_ptr: u64, length: u32, ioc: bool, cycle: bool) -> Trb {
    let mut trb = Trb::new();

    trb.parameter = buffer_ptr;
    trb.status = length;

    trb.set_type(TRB_TYPE_NORMAL);
    if ioc {
        trb.control |= 1 << 5; // IOC
    }
    trb.set_cycle(cycle);

    trb
}
