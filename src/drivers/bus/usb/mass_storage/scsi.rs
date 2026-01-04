//! # Camada de Comandos SCSI para USB Mass Storage

pub struct ScsiCommand;

impl ScsiCommand {
    pub const TEST_UNIT_READY: u8 = 0x00;
    pub const REQUEST_SENSE: u8 = 0x03;
    pub const INQUIRY: u8 = 0x12;
    pub const READ_CAPACITY_10: u8 = 0x25;
    pub const READ_10: u8 = 0x28;
    pub const WRITE_10: u8 = 0x2A;

    pub fn test_unit_ready() -> [u8; 6] {
        [0x00, 0, 0, 0, 0, 0]
    }

    pub fn request_sense(alloc_len: u8) -> [u8; 6] {
        [0x03, 0, 0, 0, alloc_len, 0]
    }

    pub fn inquiry(alloc_len: u8) -> [u8; 6] {
        [0x12, 0, 0, 0, alloc_len, 0]
    }

    pub fn read_capacity_10() -> [u8; 10] {
        [0x25, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    }

    pub fn read_10(lba: u32, count: u16) -> [u8; 10] {
        let lba_b = lba.to_be_bytes();
        let cnt_b = count.to_be_bytes();
        [
            0x28, 0, lba_b[0], lba_b[1], lba_b[2], lba_b[3], 0, cnt_b[0], cnt_b[1], 0,
        ]
    }
}

/// Estrutura de resposta do comando INQUIRY
#[repr(C, packed)]
pub struct InquiryData {
    pub device_type: u8,
    pub removable: u8,
    pub version: u8,
    pub response_format: u8,
    pub additional_len: u8,
    pub reserved: [u8; 3],
    pub vendor_id: [u8; 8],
    pub product_id: [u8; 16],
    pub product_rev: [u8; 4],
}
