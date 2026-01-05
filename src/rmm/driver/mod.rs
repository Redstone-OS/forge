//! # Driver Memory APIs
//!
//! APIs de memória para drivers de dispositivos.
//!
//! ## Componentes
//!
//! - `dma`: Alocação de buffers DMA (coherent e streaming)
//! - `iommu`: Abstração de IOMMU (VT-d, AMD-Vi)
//! - `buffer`: DeviceBuffer para I/O
//! - `pinned`: Memória pinned (não pode ser swapped/moved)
//!
//! ## Design
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │                     Driver Memory                    │
//! ├──────────────────────────────────────────────────────┤
//! │                                                      │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │
//! │  │   DMA API   │  │  IOMMU API  │  │ Pinned API  │   │
//! │  │  coherent() │  │   map()     │  │  alloc()    │   │
//! │  │  stream()   │  │   unmap()   │  │  free()     │   │
//! │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘   │
//! │         │                │                │          │
//! │         └────────────────┴────────────────┘          │
//! │                          │                           │
//! │                    ┌─────▼─────┐                     │
//! │                    │  phys::   │                     │
//! │                    │  alloc()  │                     │
//! │                    └───────────┘                     │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! ## Zonas de Memória
//!
//! - **DMA Zone** (0-16MB): ISA DMA, legacy devices
//! - **DMA32 Zone** (16MB-4GB): PCI 32-bit DMA
//! - **Normal Zone** (>4GB): Para dispositivos com 64-bit DMA ou IOMMU

pub mod buffer;
pub mod dma;
pub mod iommu;
pub mod pinned;

pub use buffer::{DeviceBuffer, DmaDirection};
pub use dma::{alloc_dma, alloc_dma_coherent, free_dma, DmaBuffer, DmaPool};
pub use iommu::{IommuDomain, IommuOps, NoIommu};
pub use pinned::{alloc_pinned, free_pinned, PinnedMemory};

use crate::rmm::addr::PhysAddr;
use crate::rmm::error::RmmResult;

/// Inicializa subsistema de memória de drivers
pub fn init() {
    crate::kinfo!("(RMM/Driver) Subsistema de memória de drivers inicializado");

    // Inicializa DMA pool
    dma::init_pool();

    // Detecta IOMMU
    iommu::detect();
}

/// Retorna se IOMMU está disponível
pub fn has_iommu() -> bool {
    iommu::is_available()
}

/// Aloca buffer para dispositivo (atalho)
pub fn alloc_device_buffer(size: usize, dma32: bool) -> RmmResult<DeviceBuffer> {
    DeviceBuffer::new(size, dma32)
}
