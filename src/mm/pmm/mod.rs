//! # PMM - Physical Memory Manager
//!
//! Gerencia alocação de frames físicos.

pub mod bitmap;
pub mod frame;
pub mod region;
pub mod stats;
pub mod zones;

pub use bitmap::BitmapFrameAllocator;
pub use frame::PhysFrame;
pub use region::{MemoryRegion, MemoryRegionType};
pub use stats::PmmStats;

use crate::sync::Mutex;

/// Tamanho de um frame (4KB)
pub const FRAME_SIZE: usize = 4096;

/// Alocador global de frames físicos
pub static FRAME_ALLOCATOR: Mutex<BitmapFrameAllocator> = Mutex::new(BitmapFrameAllocator::empty());

/// Atalho para inicializar o PMM
pub unsafe fn init(boot_info: &'static crate::core::handoff::BootInfo) {
    FRAME_ALLOCATOR.lock().init(boot_info);
}
