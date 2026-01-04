//! # Bochs Graphics Adapter (BGA) Registers
//!
//! Definições de registradores e constantes para o controlador BGA.

pub const INDEX_PORT: u16 = 0x01CE;
pub const DATA_PORT: u16 = 0x01CF;

// Registradores VBE Dispi
pub const VBE_DISPI_INDEX_ID: u16 = 0x00;
pub const VBE_DISPI_INDEX_XRES: u16 = 0x01;
pub const VBE_DISPI_INDEX_YRES: u16 = 0x02;
pub const VBE_DISPI_INDEX_BPP: u16 = 0x03;
pub const VBE_DISPI_INDEX_ENABLE: u16 = 0x04;
pub const VBE_DISPI_INDEX_BANK: u16 = 0x05;
pub const VBE_DISPI_INDEX_VIRT_WIDTH: u16 = 0x06;
pub const VBE_DISPI_INDEX_VIRT_HEIGHT: u16 = 0x07;
pub const VBE_DISPI_INDEX_X_OFFSET: u16 = 0x08;
pub const VBE_DISPI_INDEX_Y_OFFSET: u16 = 0x09;

// IDs
pub const VBE_DISPI_ID0: u16 = 0xB0C0;
pub const VBE_DISPI_ID1: u16 = 0xB0C1;
pub const VBE_DISPI_ID2: u16 = 0xB0C2;
pub const VBE_DISPI_ID3: u16 = 0xB0C3;
pub const VBE_DISPI_ID4: u16 = 0xB0C4;
pub const VBE_DISPI_ID5: u16 = 0xB0C5;

// Comandos de Enable
pub const VBE_DISPI_DISABLED: u16 = 0x00;
pub const VBE_DISPI_ENABLED: u16 = 0x01;
pub const VBE_DISPI_LFB_ENABLED: u16 = 0x40;
pub const VBE_DISPI_NOCLEARMEM: u16 = 0x80;
