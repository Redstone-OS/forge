//! # Storage Traits
//!
//! Abstrações para dispositivos de armazenamento em massa (Disco, SSD, Ramdisk).

use alloc::vec::Vec;
use core::fmt;

/// Erros de Blocos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockError {
    NotFound,
    InvalidBlock,
    IoError,
    ReadOnly,
    InvalidBuffer,
    Busy,
    HardwareError,
    NotReady,
    Unsupported,
}

/// Interface para dispositivos de bloco
pub trait BlockDevice: Send + Sync {
    /// Nome amigável do dispositivo (ex: "virtio-blk", "sda")
    fn name(&self) -> &str;

    /// Lê um único bloco do disco
    fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), BlockError>;

    /// Escreve um único bloco no disco
    fn write_block(&self, lba: u64, buf: &[u8]) -> Result<(), BlockError>;

    /// Tamanho do bloco em bytes (geralmente 512 ou 4096)
    fn block_size(&self) -> usize;

    /// Número total de blocos
    fn total_blocks(&self) -> u64;

    /// Verifica se é somente leitura
    fn is_read_only(&self) -> bool {
        false
    }
}
