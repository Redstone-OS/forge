//! Block Device - Dispositivos de bloco

use super::device::{Device, DeviceNumber, DeviceType};

/// Trait para dispositivos de bloco
pub trait BlockDevice: Device {
    /// Retorna o tamanho do bloco em bytes
    fn block_size(&self) -> usize {
        512 // Padrão: 512 bytes
    }

    /// Retorna o número total de blocos
    fn block_count(&self) -> u64;

    /// Lê um bloco
    fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), &'static str>;

    /// Escreve um bloco
    fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), &'static str>;

    /// Flush (sincroniza com disco)
    fn flush(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

/// Dispositivo de bloco base
pub struct BaseBlockDevice {
    name: &'static str,
    dev: DeviceNumber,
    block_size: usize,
    block_count: u64,
}

impl BaseBlockDevice {
    /// Cria um novo dispositivo de bloco
    pub const fn new(
        name: &'static str,
        major: u32,
        minor: u32,
        block_size: usize,
        block_count: u64,
    ) -> Self {
        Self {
            name,
            dev: DeviceNumber::new(major, minor),
            block_size,
            block_count,
        }
    }
}

impl Device for BaseBlockDevice {
    fn name(&self) -> &str {
        self.name
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }

    fn device_number(&self) -> DeviceNumber {
        self.dev
    }
}

impl BlockDevice for BaseBlockDevice {
    fn block_size(&self) -> usize {
        self.block_size
    }

    fn block_count(&self) -> u64 {
        self.block_count
    }

    fn read_block(&self, _block: u64, _buf: &mut [u8]) -> Result<(), &'static str> {
        Err("Not implemented")
    }

    fn write_block(&self, _block: u64, _buf: &[u8]) -> Result<(), &'static str> {
        Err("Not implemented")
    }
}
