//! # Driver VirtIO Block Device
//!
//! Implementa virtio-blk para discos virtuais do QEMU.
//!
//! ## Referências
//!
//! - [Especificação VirtIO 1.1](https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html)
//! - Dispositivo virtio-blk do QEMU
//!
//! ## Funcionamento
//!
//! O VirtIO é um padrão de paravirtualização que permite comunicação
//! eficiente entre guest e host. O dispositivo aparece no barramento PCI
//! com vendor=0x1AF4 (Red Hat/Virtio) e device=0x1001 (block).

#![allow(dead_code)]

use super::traits::{BlockDevice, BlockError};
use crate::mm::VirtAddr;
use crate::sync::Spinlock;
use alloc::sync::Arc;

/// Tamanho padrão de setor
const SECTOR_SIZE: usize = 512;

/// Dispositivo de Bloco VirtIO
pub struct VirtioBlk {
    /// Endereço base MMIO
    base: VirtAddr,
    /// Total de setores
    total_sectors: u64,
    /// Lock do dispositivo
    lock: Spinlock<()>,
}

// SAFETY: VirtioBlk usa locking interno
unsafe impl Send for VirtioBlk {}
unsafe impl Sync for VirtioBlk {}

impl VirtioBlk {
    /// Cria um novo dispositivo de bloco VirtIO
    pub fn new(base: VirtAddr, total_sectors: u64) -> Self {
        Self {
            base,
            total_sectors,
            lock: Spinlock::new(()),
        }
    }
}

impl BlockDevice for VirtioBlk {
    fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        if lba >= self.total_sectors {
            return Err(BlockError::InvalidBlock);
        }
        if buf.len() < SECTOR_SIZE {
            return Err(BlockError::InvalidBuffer);
        }

        let _guard = self.lock.lock();

        // TODO: Implementar leitura real via virtqueue
        crate::ktrace!("(VirtIO) Lendo setor:", lba);

        Err(BlockError::NotFound)
    }

    fn write_block(&self, lba: u64, buf: &[u8]) -> Result<(), BlockError> {
        if lba >= self.total_sectors {
            return Err(BlockError::InvalidBlock);
        }
        if buf.len() < SECTOR_SIZE {
            return Err(BlockError::InvalidBuffer);
        }

        let _guard = self.lock.lock();

        // TODO: Implementar escrita real via virtqueue
        crate::ktrace!("(VirtIO) Escrevendo setor:", lba);

        Err(BlockError::NotFound)
    }

    fn block_size(&self) -> usize {
        SECTOR_SIZE
    }

    fn total_blocks(&self) -> u64 {
        self.total_sectors
    }
}

/// Tenta inicializar dispositivo virtio-blk
///
/// Retorna None se nenhum dispositivo for encontrado.
pub fn init() -> Option<Arc<dyn BlockDevice>> {
    crate::kinfo!("(VirtIO-BLK) Procurando dispositivo...");

    // TODO: Escanear por dispositivo virtio-blk via:
    // 1. Tabelas ACPI (QEMU fornece informações do device tree)
    // 2. Enumeração PCI (virtio-pci)
    // 3. Descoberta MMIO (virtio-mmio)
    //
    // Para QEMU com `-drive if=virtio`, o dispositivo aparece como PCI
    // com vendor=0x1AF4 (Red Hat/Virtio) e device=0x1001 (block)

    // Por enquanto, retorna None pois precisamos de enumeração PCI
    crate::kwarn!("(VirtIO-BLK) Nenhum dispositivo encontrado (PCI não implementado)");
    None
}
