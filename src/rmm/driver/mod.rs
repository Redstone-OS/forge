//! # Driver Memory API
//!
//! API de memória para drivers: DMA buffers, IOMMU, pinned memory.

pub mod buffer;
pub mod dma;
pub mod iommu;
pub mod pinned;

pub use buffer::DeviceBuffer;
pub use dma::{alloc_dma, free_dma, DmaBuffer, DmaDirection};
pub use pinned::{alloc_pinned, free_pinned};
