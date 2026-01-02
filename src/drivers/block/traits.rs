//! # Camada de Abstração de Dispositivos de Bloco
//!
//! Fornece traits e tipos para drivers de dispositivos de bloco.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │              FILESYSTEM (FAT, RFS)                  │
//! └─────────────────────────────────────────────────────┘
//!                          ↓
//! ┌─────────────────────────────────────────────────────┐
//! │              BlockDevice Trait                      │
//! │   read_block() write_block() block_size()          │
//! └─────────────────────────────────────────────────────┘
//!                          ↓
//! ┌─────────────────────────────────────────────────────┐
//! │              DRIVERS (VirtIO, AHCI, NVMe)           │
//! └─────────────────────────────────────────────────────┘
//! ```

use core::fmt;

/// Tipos de erro para dispositivos de bloco
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockError {
    /// Dispositivo não encontrado ou não inicializado
    NotFound,
    /// Endereço de bloco inválido (fora do intervalo)
    InvalidBlock,
    /// Erro de I/O durante leitura/escrita
    IoError,
    /// Dispositivo somente leitura
    ReadOnly,
    /// Tamanho do buffer incorreto
    InvalidBuffer,
    /// Dispositivo ocupado
    Busy,
    /// Erro genérico de hardware
    HardwareError,
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockError::NotFound => write!(f, "Dispositivo não encontrado"),
            BlockError::InvalidBlock => write!(f, "Endereço de bloco inválido"),
            BlockError::IoError => write!(f, "Erro de I/O"),
            BlockError::ReadOnly => write!(f, "Dispositivo somente leitura"),
            BlockError::InvalidBuffer => write!(f, "Tamanho do buffer inválido"),
            BlockError::Busy => write!(f, "Dispositivo ocupado"),
            BlockError::HardwareError => write!(f, "Erro de hardware"),
        }
    }
}

/// Trait para dispositivos de bloco
///
/// Todos os drivers de dispositivos de bloco devem implementar esta trait.
///
/// # Exemplo
///
/// ```ignore
/// let device = get_device(0).unwrap();
/// let mut buffer = [0u8; 512];
/// device.read_block(0, &mut buffer)?;
/// ```
pub trait BlockDevice: Send + Sync {
    /// Lê um único bloco do dispositivo
    ///
    /// # Argumentos
    /// * `lba` - Endereço Lógico de Bloco (Logical Block Address)
    /// * `buf` - Buffer para armazenar os dados (mínimo block_size bytes)
    fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), BlockError>;

    /// Escreve um único bloco no dispositivo
    ///
    /// # Argumentos
    /// * `lba` - Endereço Lógico de Bloco
    /// * `buf` - Buffer com os dados a escrever (mínimo block_size bytes)
    fn write_block(&self, lba: u64, buf: &[u8]) -> Result<(), BlockError>;

    /// Retorna o tamanho do bloco em bytes (normalmente 512)
    fn block_size(&self) -> usize;

    /// Retorna o número total de blocos no dispositivo
    fn total_blocks(&self) -> u64;

    /// Verifica se o dispositivo é somente leitura
    fn is_read_only(&self) -> bool {
        false
    }

    /// Força a escrita de dados em cache para o dispositivo
    fn flush(&self) -> Result<(), BlockError> {
        Ok(())
    }

    /// Lê múltiplos blocos contíguos
    fn read_blocks(&self, start_lba: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        let block_size = self.block_size();
        if buf.len() % block_size != 0 {
            return Err(BlockError::InvalidBuffer);
        }

        let num_blocks = buf.len() / block_size;
        for i in 0..num_blocks {
            let offset = i * block_size;
            self.read_block(start_lba + i as u64, &mut buf[offset..offset + block_size])?;
        }
        Ok(())
    }

    /// Escreve múltiplos blocos contíguos
    fn write_blocks(&self, start_lba: u64, buf: &[u8]) -> Result<(), BlockError> {
        let block_size = self.block_size();
        if buf.len() % block_size != 0 {
            return Err(BlockError::InvalidBuffer);
        }

        let num_blocks = buf.len() / block_size;
        for i in 0..num_blocks {
            let offset = i * block_size;
            self.write_block(start_lba + i as u64, &buf[offset..offset + block_size])?;
        }
        Ok(())
    }
}

/// Informações sobre um dispositivo de bloco
#[derive(Debug, Clone)]
pub struct BlockDeviceInfo {
    /// Nome do dispositivo (ex: "virtio0", "nvme0n1")
    pub name: &'static str,
    /// Tamanho do bloco em bytes
    pub block_size: usize,
    /// Número total de blocos
    pub total_blocks: u64,
    /// Se o dispositivo é somente leitura
    pub read_only: bool,
}

impl BlockDeviceInfo {
    /// Calcula o tamanho total em bytes
    pub fn size_bytes(&self) -> u64 {
        self.total_blocks * self.block_size as u64
    }

    /// Calcula o tamanho em MB
    pub fn size_mb(&self) -> u64 {
        self.size_bytes() / (1024 * 1024)
    }
}
