//! # Operações PIO do ATA
//!
//! Implementação das leituras e escritas usando PIO (Programmed I/O).

use super::io::{inb, inw, io_delay, outb};
use super::ports::{commands, primary as ports, status};
use crate::drivers::block::traits::BlockError;

/// Tamanho de um setor em bytes
pub const SECTOR_SIZE: usize = 512;

/// Timeout padrão para operações (número de iterações)
const TIMEOUT: u32 = 100_000;

/// Espera o drive ficar pronto (BSY=0)
pub fn wait_ready() -> bool {
    for _ in 0..TIMEOUT {
        let s = unsafe { inb(ports::STATUS) };
        if s & status::BSY == 0 {
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

/// Espera dados disponíveis (DRQ=1)
pub fn wait_drq() -> bool {
    for _ in 0..TIMEOUT {
        let s = unsafe { inb(ports::STATUS) };
        if s & status::ERR != 0 {
            return false;
        }
        if s & status::DRQ != 0 {
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

/// Verifica se há erro
#[allow(dead_code)]
pub fn check_error() -> Option<u8> {
    let s = unsafe { inb(ports::STATUS) };
    if s & status::ERR != 0 {
        Some(unsafe { inb(ports::ERROR) })
    } else {
        None
    }
}

/// Seleciona um drive (master=0, slave=1)
pub fn select_drive(drive: u8) {
    let value = if drive == 0 { 0xA0 } else { 0xB0 };
    unsafe {
        outb(ports::DRIVE_HEAD, value);
    }
    io_delay();
}

/// Executa o comando IDENTIFY para obter informações do drive
///
/// Retorna um buffer de 256 words (512 bytes) com informações do disco.
pub fn identify(drive: u8) -> Option<[u16; 256]> {
    // Selecionar drive
    select_drive(drive);

    if !wait_ready() {
        return None;
    }

    // Enviar comando IDENTIFY
    unsafe { outb(ports::COMMAND, commands::IDENTIFY) };

    // Verificar se drive existe
    let s = unsafe { inb(ports::STATUS) };
    if s == 0 {
        return None;
    }

    // Esperar dados prontos
    if !wait_drq() {
        return None;
    }

    // Ler 256 words (512 bytes) de identificação
    let mut data = [0u16; 256];
    for word in data.iter_mut() {
        *word = unsafe { inw(ports::DATA) };
    }

    Some(data)
}

/// Lê setores usando PIO LBA28
///
/// # Arguments
/// * `lba` - Endereço Lógico do Bloco (máximo 0x0FFFFFFF para LBA28)
/// * `count` - Número de setores a ler (1-255, 0 significa 256)
/// * `buf` - Buffer para armazenar os dados
pub fn read_sectors_lba28(lba: u64, count: u8, buf: &mut [u8]) -> Result<(), BlockError> {
    // Verificar limite LBA28
    if lba > 0x0FFF_FFFF {
        return Err(BlockError::InvalidBlock);
    }

    // Verificar tamanho do buffer
    let expected_size = (count as usize) * SECTOR_SIZE;
    if buf.len() < expected_size {
        return Err(BlockError::InvalidBuffer);
    }

    let lba = lba as u32;

    unsafe {
        // Esperar drive pronto
        if !wait_ready() {
            return Err(BlockError::IoError);
        }

        // Configurar LBA e drive (LBA mode)
        outb(ports::DRIVE_HEAD, 0xE0 | ((lba >> 24) & 0x0F) as u8);
        outb(ports::SECTOR_COUNT, count);
        outb(ports::LBA_LO, (lba & 0xFF) as u8);
        outb(ports::LBA_MID, ((lba >> 8) & 0xFF) as u8);
        outb(ports::LBA_HI, ((lba >> 16) & 0xFF) as u8);

        // Enviar comando READ
        outb(ports::COMMAND, commands::READ_SECTORS);

        // Ler cada setor
        for sector in 0..count as usize {
            if !wait_drq() {
                return Err(BlockError::IoError);
            }

            // Ler 256 words (512 bytes)
            let offset = sector * SECTOR_SIZE;
            for i in (0..SECTOR_SIZE).step_by(2) {
                let word = inw(ports::DATA);
                buf[offset + i] = (word & 0xFF) as u8;
                buf[offset + i + 1] = ((word >> 8) & 0xFF) as u8;
            }
        }
    }

    Ok(())
}

/// Escreve setores usando PIO LBA28
///
/// # Arguments
/// * `lba` - Endereço Lógico do Bloco (máximo 0x0FFFFFFF para LBA28)
/// * `count` - Número de setores a escrever (1-255, 0 significa 256)
/// * `buf` - Buffer com os dados a escrever
#[allow(dead_code)]
pub fn write_sectors_lba28(lba: u64, count: u8, buf: &[u8]) -> Result<(), BlockError> {
    use super::io::outw;

    // Verificar limite LBA28
    if lba > 0x0FFF_FFFF {
        return Err(BlockError::InvalidBlock);
    }

    // Verificar tamanho do buffer
    let expected_size = (count as usize) * SECTOR_SIZE;
    if buf.len() < expected_size {
        return Err(BlockError::InvalidBuffer);
    }

    let lba = lba as u32;

    unsafe {
        // Esperar drive pronto
        if !wait_ready() {
            return Err(BlockError::IoError);
        }

        // Configurar LBA e drive (LBA mode)
        outb(ports::DRIVE_HEAD, 0xE0 | ((lba >> 24) & 0x0F) as u8);
        outb(ports::SECTOR_COUNT, count);
        outb(ports::LBA_LO, (lba & 0xFF) as u8);
        outb(ports::LBA_MID, ((lba >> 8) & 0xFF) as u8);
        outb(ports::LBA_HI, ((lba >> 16) & 0xFF) as u8);

        // Enviar comando WRITE
        outb(ports::COMMAND, commands::WRITE_SECTORS);

        // Escrever cada setor
        for sector in 0..count as usize {
            if !wait_drq() {
                return Err(BlockError::IoError);
            }

            // Escrever 256 words (512 bytes)
            let offset = sector * SECTOR_SIZE;
            for i in (0..SECTOR_SIZE).step_by(2) {
                let word = (buf[offset + i] as u16) | ((buf[offset + i + 1] as u16) << 8);
                outw(ports::DATA, word);
            }

            // Delay após cada setor
            io_delay();
        }

        // Flush cache
        outb(ports::COMMAND, commands::FLUSH_CACHE);
        if !wait_ready() {
            return Err(BlockError::IoError);
        }
    }

    Ok(())
}
