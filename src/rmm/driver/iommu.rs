//! # IOMMU Abstraction
//!
//! Trait e stubs para IOMMU.

use super::dma::DmaDirection;
use crate::rmm::addr::PhysAddr;

/// Endereço DMA (o que o dispositivo vê)
#[derive(Debug, Clone, Copy)]
pub struct DmaAddr(pub u64);

/// Trait para backends de IOMMU
pub trait IommuOps: Send + Sync {
    /// Mapeia região física no domínio do dispositivo
    fn map(&self, phys: PhysAddr, size: usize, dir: DmaDirection) -> DmaAddr;
    /// Remove mapeamento
    fn unmap(&self, dma: DmaAddr, size: usize);
    /// Sincroniza cache para CPU
    fn sync_for_cpu(&self, dma: DmaAddr, size: usize);
    /// Sincroniza cache para device
    fn sync_for_device(&self, dma: DmaAddr, size: usize);
}

/// Implementação stub (sem IOMMU real)
pub struct NoIommu;

impl IommuOps for NoIommu {
    fn map(&self, phys: PhysAddr, _size: usize, _dir: DmaDirection) -> DmaAddr {
        // Sem IOMMU: dma == phys
        DmaAddr(phys.as_u64())
    }

    fn unmap(&self, _dma: DmaAddr, _size: usize) {
        // Nada a fazer
    }

    fn sync_for_cpu(&self, _dma: DmaAddr, _size: usize) {
        // x86 é cache-coherent
    }

    fn sync_for_device(&self, _dma: DmaAddr, _size: usize) {
        // x86 é cache-coherent
    }
}

/// IOMMU atual (global)
static mut CURRENT_IOMMU: Option<&'static dyn IommuOps> = None;

/// Retorna IOMMU atual
pub fn get_iommu() -> &'static dyn IommuOps {
    static NO_IOMMU: NoIommu = NoIommu;
    unsafe { CURRENT_IOMMU.unwrap_or(&NO_IOMMU) }
}
