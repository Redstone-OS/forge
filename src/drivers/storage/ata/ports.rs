//! # Portas e Constantes ATA/IDE
//!
//! Definições de portas I/O e constantes do controlador ATA.

/// Portas do Primary ATA Controller
pub mod primary {
    /// Data Register - Leitura/Escrita de dados (16 bits)
    pub const DATA: u16 = 0x1F0;
    /// Error Register (leitura) / Features Register (escrita)
    pub const ERROR: u16 = 0x1F1;
    /// Features Register (escrita)
    pub const FEATURES: u16 = 0x1F1;
    /// Sector Count Register
    pub const SECTOR_COUNT: u16 = 0x1F2;
    /// LBA Low (bits 0-7 do LBA)
    pub const LBA_LO: u16 = 0x1F3;
    /// LBA Mid (bits 8-15 do LBA)
    pub const LBA_MID: u16 = 0x1F4;
    /// LBA High (bits 16-23 do LBA)
    pub const LBA_HI: u16 = 0x1F5;
    /// Drive/Head Register
    pub const DRIVE_HEAD: u16 = 0x1F6;
    /// Status Register (leitura)
    pub const STATUS: u16 = 0x1F7;
    /// Command Register (escrita)
    pub const COMMAND: u16 = 0x1F7;
    /// Alternate Status Register (leitura, não limpa IRQ)
    pub const ALT_STATUS: u16 = 0x3F6;
    /// Device Control Register (escrita)
    pub const DEV_CONTROL: u16 = 0x3F6;
}

/// Portas do Secondary ATA Controller
#[allow(dead_code)]
pub mod secondary {
    pub const DATA: u16 = 0x170;
    pub const ERROR: u16 = 0x171;
    pub const FEATURES: u16 = 0x171;
    pub const SECTOR_COUNT: u16 = 0x172;
    pub const LBA_LO: u16 = 0x173;
    pub const LBA_MID: u16 = 0x174;
    pub const LBA_HI: u16 = 0x175;
    pub const DRIVE_HEAD: u16 = 0x176;
    pub const STATUS: u16 = 0x177;
    pub const COMMAND: u16 = 0x177;
    pub const ALT_STATUS: u16 = 0x376;
    pub const DEV_CONTROL: u16 = 0x376;
}

/// Bits do Status Register
pub mod status {
    /// Erro ocorreu
    pub const ERR: u8 = 0x01;
    /// Index (sempre 0)
    pub const IDX: u8 = 0x02;
    /// Corrected data (obsoleto)
    pub const CORR: u8 = 0x04;
    /// Data Request Ready
    pub const DRQ: u8 = 0x08;
    /// Drive Seek Complete (obsoleto)
    pub const DSC: u8 = 0x10;
    /// Drive Write Fault
    pub const DWF: u8 = 0x20;
    /// Drive Ready
    pub const DRDY: u8 = 0x40;
    /// Drive Busy
    pub const BSY: u8 = 0x80;
}

/// Comandos ATA
pub mod commands {
    /// NOP
    pub const NOP: u8 = 0x00;
    /// Lê setores (PIO, LBA28)
    pub const READ_SECTORS: u8 = 0x20;
    /// Lê setores (PIO, LBA48)
    pub const READ_SECTORS_EXT: u8 = 0x24;
    /// Escreve setores (PIO, LBA28)
    pub const WRITE_SECTORS: u8 = 0x30;
    /// Escreve setores (PIO, LBA48)
    pub const WRITE_SECTORS_EXT: u8 = 0x34;
    /// Cache flush
    pub const FLUSH_CACHE: u8 = 0xE7;
    /// Identify Device - retorna informações do disco
    pub const IDENTIFY: u8 = 0xEC;
    /// Set Features
    pub const SET_FEATURES: u8 = 0xEF;
}

/// Valores do Drive/Head Register
pub mod drive {
    /// Selecionar Master (drive 0)
    pub const MASTER: u8 = 0xA0;
    /// Selecionar Slave (drive 1)
    pub const SLAVE: u8 = 0xB0;
    /// Modo LBA ativado
    pub const LBA: u8 = 0x40;
}

/// Bits de Device Control
pub mod control {
    /// High Order Byte (usado para LBA48)
    pub const HOB: u8 = 0x80;
    /// Disable Interrupts
    pub const NIEN: u8 = 0x02;
    /// Software Reset
    pub const SRST: u8 = 0x04;
}

/// Campos interessantes do buffer IDENTIFY
pub mod identify_offsets {
    /// Modelo do dispositivo (20 words, offset 27)
    pub const MODEL: usize = 27;
    /// Número de setores LBA28 (2 words, offset 60)
    pub const SECTORS_28: usize = 60;
    /// Suporte a comandos (word 83)
    pub const COMMAND_SET_SUPPORT: usize = 83;
    /// Número de setores LBA48 (4 words, offset 100)
    pub const SECTORS_48: usize = 100;
}
