//! # SCSI Commands for USB Mass Storage
//!
//! Comandos SCSI usados pelo USB Mass Storage.

// =============================================================================
// SCSI OPCODES
// =============================================================================

pub const SCSI_TEST_UNIT_READY: u8 = 0x00;
pub const SCSI_REQUEST_SENSE: u8 = 0x03;
pub const SCSI_INQUIRY: u8 = 0x12;
pub const SCSI_MODE_SELECT_6: u8 = 0x15;
pub const SCSI_MODE_SENSE_6: u8 = 0x1A;
pub const SCSI_START_STOP_UNIT: u8 = 0x1B;
pub const SCSI_PREVENT_ALLOW_MEDIUM_REMOVAL: u8 = 0x1E;
pub const SCSI_READ_CAPACITY_10: u8 = 0x25;
pub const SCSI_READ_10: u8 = 0x28;
pub const SCSI_WRITE_10: u8 = 0x2A;
pub const SCSI_READ_CAPACITY_16: u8 = 0x9E;
pub const SCSI_READ_16: u8 = 0x88;
pub const SCSI_WRITE_16: u8 = 0x8A;

// =============================================================================
// CDB (COMMAND DESCRIPTOR BLOCK)
// =============================================================================

/// CDB de 6 bytes (comandos simples).
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct Cdb6 {
    pub opcode: u8,
    pub lba_hi: u8,
    pub lba_mid: u8,
    pub lba_lo: u8,
    pub length: u8,
    pub control: u8,
}

/// CDB de 10 bytes (comandos mais comuns).
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct Cdb10 {
    pub opcode: u8,
    pub flags: u8,
    pub lba: [u8; 4], // Big-endian
    pub group: u8,
    pub length: [u8; 2], // Big-endian
    pub control: u8,
}

/// CDB de 16 bytes (comandos para discos grandes).
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct Cdb16 {
    pub opcode: u8,
    pub flags: u8,
    pub lba: [u8; 8],    // Big-endian
    pub length: [u8; 4], // Big-endian
    pub group: u8,
    pub control: u8,
}

// =============================================================================
// INQUIRY DATA
// =============================================================================

/// Resposta do comando INQUIRY.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct InquiryData {
    pub peripheral: u8, // Device type
    pub removable: u8,  // Bit 7 = removable
    pub version: u8,    // SCSI version
    pub response_format: u8,
    pub additional_length: u8,
    pub reserved: [u8; 3],
    pub vendor_id: [u8; 8],
    pub product_id: [u8; 16],
    pub revision: [u8; 4],
}

// =============================================================================
// READ CAPACITY
// =============================================================================

/// Resposta do comando READ CAPACITY (10).
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct ReadCapacity10Data {
    pub last_lba: [u8; 4],     // Big-endian
    pub block_length: [u8; 4], // Big-endian
}

impl ReadCapacity10Data {
    pub fn last_lba(&self) -> u32 {
        u32::from_be_bytes(self.last_lba)
    }

    pub fn block_length(&self) -> u32 {
        u32::from_be_bytes(self.block_length)
    }
}

// =============================================================================
// REQUEST SENSE
// =============================================================================

/// Resposta do comando REQUEST SENSE.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct SenseData {
    pub response_code: u8,
    pub segment_number: u8,
    pub sense_key: u8,
    pub information: [u8; 4],
    pub additional_length: u8,
    pub command_specific: [u8; 4],
    pub asc: u8,  // Additional Sense Code
    pub ascq: u8, // Additional Sense Code Qualifier
    pub field_replaceable: u8,
    pub sense_key_specific: [u8; 3],
}

impl SenseData {
    pub fn sense_key(&self) -> u8 {
        self.sense_key & 0x0F
    }
}

// Sense Keys
pub const SENSE_NO_SENSE: u8 = 0x00;
pub const SENSE_RECOVERED_ERROR: u8 = 0x01;
pub const SENSE_NOT_READY: u8 = 0x02;
pub const SENSE_MEDIUM_ERROR: u8 = 0x03;
pub const SENSE_HARDWARE_ERROR: u8 = 0x04;
pub const SENSE_ILLEGAL_REQUEST: u8 = 0x05;
pub const SENSE_UNIT_ATTENTION: u8 = 0x06;
pub const SENSE_DATA_PROTECT: u8 = 0x07;
pub const SENSE_BLANK_CHECK: u8 = 0x08;
pub const SENSE_ABORTED_COMMAND: u8 = 0x0B;

// =============================================================================
// FUNÇÕES DE CONSTRUÇÃO DE CDB
// =============================================================================

/// Cria CDB para TEST UNIT READY.
pub fn test_unit_ready() -> Cdb6 {
    Cdb6 {
        opcode: SCSI_TEST_UNIT_READY,
        ..Default::default()
    }
}

/// Cria CDB para INQUIRY.
pub fn inquiry(allocation_length: u8) -> Cdb6 {
    Cdb6 {
        opcode: SCSI_INQUIRY,
        length: allocation_length,
        ..Default::default()
    }
}

/// Cria CDB para READ CAPACITY (10).
pub fn read_capacity_10() -> Cdb10 {
    Cdb10 {
        opcode: SCSI_READ_CAPACITY_10,
        ..Default::default()
    }
}

/// Cria CDB para READ (10).
pub fn read_10(lba: u32, block_count: u16) -> Cdb10 {
    Cdb10 {
        opcode: SCSI_READ_10,
        lba: lba.to_be_bytes(),
        length: block_count.to_be_bytes(),
        ..Default::default()
    }
}

/// Cria CDB para WRITE (10).
pub fn write_10(lba: u32, block_count: u16) -> Cdb10 {
    Cdb10 {
        opcode: SCSI_WRITE_10,
        lba: lba.to_be_bytes(),
        length: block_count.to_be_bytes(),
        ..Default::default()
    }
}

/// Cria CDB para REQUEST SENSE.
pub fn request_sense(allocation_length: u8) -> Cdb6 {
    Cdb6 {
        opcode: SCSI_REQUEST_SENSE,
        length: allocation_length,
        ..Default::default()
    }
}
