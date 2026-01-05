//! # DMA Buffer Allocation
//!
//! Alocação de buffers para DMA com endereços físicos contíguos.

use crate::rmm::addr::PhysAddr;

/// Direção do DMA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    /// CPU → Device
    ToDevice,
    /// Device → CPU
    FromDevice,
    /// Bidirecional
    Bidirectional,
}

/// Buffer DMA com endereços físico, virtual e DMA
pub struct DmaBuffer {
    /// Endereço virtual (acesso CPU)
    pub virt: u64,
    /// Endereço físico (RAM real)
    pub phys: u64,
    /// Endereço DMA (o que o device vê)
    pub dma: u64,
    /// Tamanho
    pub size: usize,
    /// Direção
    pub direction: DmaDirection,
    /// Owner (device ID)
    pub owner: u32,
}

impl DmaBuffer {
    /// Retorna slice para escrita
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        core::slice::from_raw_parts_mut(self.virt as *mut u8, self.size)
    }

    /// Retorna slice para leitura
    pub unsafe fn as_slice(&self) -> &[u8] {
        core::slice::from_raw_parts(self.virt as *const u8, self.size)
    }
}

/// Aloca buffer DMA
pub fn alloc_dma(size: usize, direction: DmaDirection) -> Option<DmaBuffer> {
    // TODO: Implementar
    // 1. Alocar frames contíguos na zona DMA32
    // 2. Mapear com NO_CACHE
    // 3. Configurar IOMMU se disponível
    None
}

/// Libera buffer DMA
pub fn free_dma(buffer: DmaBuffer) {
    // TODO: Implementar
    // 1. Desmapear IOMMU
    // 2. Desmapear virtual
    // 3. Liberar frames físicos
}
