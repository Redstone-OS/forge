// Módulos
pub mod addr;
pub mod alloc;
pub mod heap;
pub mod oom;
pub mod ops;
pub mod pmm;
pub mod test;
pub mod vmm;

// Configuração compartilhada
pub mod config;
pub mod error;

// Re-exports úteis do PMM (que agora vive em pmm/mod.rs -> bitmap, frame, etc)
pub use pmm::init;
pub use pmm::{
    BitmapFrameAllocator, MemoryRegion, MemoryRegionType, PhysFrame, PmmStats, FRAME_ALLOCATOR,
}; // Atalho para inicialização

// Re-exports de addr
pub use addr::{PhysAddr, VirtAddr};

// Error
pub use error::MmError;
pub type Result<T> = core::result::Result<T, MmError>;
