//! # Driver ATA/IDE
//!
//! Driver simples para controlador ATA/IDE (modo PIO).
//!
//! ## Portas I/O
//!
//! | Porta  | Função           |
//! |--------|------------------|
//! | 0x1F0  | Data Register    |
//! | 0x1F1  | Error/Features   |
//! | 0x1F2  | Sector Count     |
//! | 0x1F3  | LBA Low          |
//! | 0x1F4  | LBA Mid          |
//! | 0x1F5  | LBA High         |
//! | 0x1F6  | Drive/Head       |
//! | 0x1F7  | Status/Command   |

#![allow(dead_code)]

use super::traits::{BlockDevice, BlockError};
use alloc::sync::Arc;
use core::arch::asm;

/// Portas do Primary ATA
mod ports {
    pub const DATA: u16 = 0x1F0;
    pub const ERROR: u16 = 0x1F1;
    pub const SECTOR_COUNT: u16 = 0x1F2;
    pub const LBA_LO: u16 = 0x1F3;
    pub const LBA_MID: u16 = 0x1F4;
    pub const LBA_HI: u16 = 0x1F5;
    pub const DRIVE_HEAD: u16 = 0x1F6;
    pub const STATUS: u16 = 0x1F7;
    pub const COMMAND: u16 = 0x1F7;
}

/// Bits do Status Register
mod status {
    pub const BSY: u8 = 0x80; // Busy
    pub const DRDY: u8 = 0x40; // Drive Ready
    pub const DRQ: u8 = 0x08; // Data Request
    pub const ERR: u8 = 0x01; // Error
}

/// Comandos ATA
mod cmd {
    pub const READ_SECTORS: u8 = 0x20;
    pub const WRITE_SECTORS: u8 = 0x30;
    pub const IDENTIFY: u8 = 0xEC;
}

/// Driver ATA
pub struct AtaDrive {
    /// 0 = master, 1 = slave
    drive: u8,
    /// Número total de setores
    sectors: u64,
}

impl AtaDrive {
    /// Inicializa o drive ATA primary master
    pub fn new() -> Option<Self> {
        crate::kinfo!("(ATA) Inicializando Primary Master...");

        // Selecionar drive master
        unsafe { outb(ports::DRIVE_HEAD, 0xA0) };

        // Esperar drive ficar pronto
        if !wait_ready() {
            crate::kwarn!("(ATA) Drive não respondeu");
            return None;
        }

        // Enviar comando IDENTIFY
        unsafe { outb(ports::COMMAND, cmd::IDENTIFY) };

        // Verificar se drive existe
        let status = unsafe { inb(ports::STATUS) };
        if status == 0 {
            crate::kwarn!("(ATA) Nenhum drive detectado");
            return None;
        }

        // Esperar dados prontos
        if !wait_drq() {
            crate::kwarn!("(ATA) Timeout esperando dados IDENTIFY");
            return None;
        }

        // Ler 256 words (512 bytes) de identificação
        let mut identify = [0u16; 256];
        for word in identify.iter_mut() {
            *word = unsafe { inw(ports::DATA) };
        }

        // Extrair número de setores (LBA28 ou LBA48)
        let sectors = if identify[83] & (1 << 10) != 0 {
            // LBA48 suportado
            (identify[100] as u64)
                | ((identify[101] as u64) << 16)
                | ((identify[102] as u64) << 32)
                | ((identify[103] as u64) << 48)
        } else {
            // LBA28 apenas
            (identify[60] as u64) | ((identify[61] as u64) << 16)
        };

        crate::kinfo!("(ATA) Drive detectado!");
        crate::kinfo!("(ATA) Setores:", sectors);
        crate::kinfo!("(ATA) Capacidade MB:", (sectors * 512) / (1024 * 1024));

        Some(Self { drive: 0, sectors })
    }

    /// Lê setores usando PIO LBA28
    fn read_sectors_pio(&self, lba: u64, count: u8, buf: &mut [u8]) -> Result<(), BlockError> {
        if lba > 0x0FFFFFFF {
            return Err(BlockError::InvalidBlock);
        }

        let lba = lba as u32;

        unsafe {
            // Esperar drive pronto
            if !wait_ready() {
                return Err(BlockError::IoError);
            }

            // Configurar LBA e drive
            outb(ports::DRIVE_HEAD, 0xE0 | ((lba >> 24) & 0x0F) as u8);
            outb(ports::SECTOR_COUNT, count);
            outb(ports::LBA_LO, (lba & 0xFF) as u8);
            outb(ports::LBA_MID, ((lba >> 8) & 0xFF) as u8);
            outb(ports::LBA_HI, ((lba >> 16) & 0xFF) as u8);

            // Enviar comando READ
            outb(ports::COMMAND, cmd::READ_SECTORS);

            // Ler cada setor
            for sector in 0..count as usize {
                if !wait_drq() {
                    return Err(BlockError::IoError);
                }

                // Ler 256 words (512 bytes)
                let offset = sector * 512;
                for i in (0..512).step_by(2) {
                    let word = inw(ports::DATA);
                    buf[offset + i] = (word & 0xFF) as u8;
                    buf[offset + i + 1] = ((word >> 8) & 0xFF) as u8;
                }
            }
        }

        Ok(())
    }
}

impl BlockDevice for AtaDrive {
    fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        if buf.len() < 512 {
            return Err(BlockError::InvalidBuffer);
        }
        self.read_sectors_pio(block, 1, buf)
    }

    fn write_block(&self, _block: u64, _buf: &[u8]) -> Result<(), BlockError> {
        // TODO: Implementar escrita
        Err(BlockError::ReadOnly)
    }

    fn block_size(&self) -> usize {
        512
    }

    fn total_blocks(&self) -> u64 {
        self.sectors
    }

    fn is_read_only(&self) -> bool {
        // Por enquanto, somente leitura
        true
    }
}

/// Espera o drive ficar pronto (BSY=0)
fn wait_ready() -> bool {
    for _ in 0..100000 {
        let status = unsafe { inb(ports::STATUS) };
        if status & status::BSY == 0 {
            return true;
        }
    }
    false
}

/// Espera dados disponíveis (DRQ=1)
fn wait_drq() -> bool {
    for _ in 0..100000 {
        let status = unsafe { inb(ports::STATUS) };
        if status & status::ERR != 0 {
            return false;
        }
        if status & status::DRQ != 0 {
            return true;
        }
    }
    false
}

// I/O helpers
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    value
}

unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}

unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack));
    value
}

/// Inicializa o driver ATA e retorna o dispositivo se encontrado
pub fn init() -> Option<Arc<dyn BlockDevice>> {
    AtaDrive::new().map(|d| Arc::new(d) as Arc<dyn BlockDevice>)
}
