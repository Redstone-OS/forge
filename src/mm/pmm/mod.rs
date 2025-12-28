/// Physical Memory Manager (PMM)
///
/// Gerencia alocação de frames físicos.
/// Physical Memory Manager (PMM)
///
/// Gerencia alocação de frames físicos.
pub mod bitmap;
pub use bitmap::BitmapFrameAllocator;
pub mod pt_scanner;
pub mod region;
pub mod stats;

use crate::mm::addr::PhysAddr;
use crate::mm::error::MmResult;
use crate::sync::Spinlock;

/// Trait para alocadores de frames físicos
pub trait FrameAllocator {
    fn allocate_frame(&self) -> Option<PhysAddr>;
    fn deallocate_frame(&self, frame: PhysAddr);
}

impl FrameAllocator for BitmapFrameAllocator {
    fn allocate_frame(&self) -> Option<PhysAddr> {
        self.allocate_frame()
    }

    fn deallocate_frame(&self, frame: PhysAddr) {
        self.deallocate_frame(frame)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysFrame(PhysAddr);

impl PhysFrame {
    pub fn containing_address(addr: PhysAddr) -> Self {
        Self(PhysAddr::new(addr.as_u64() & !(FRAME_SIZE - 1)))
    }

    pub fn from_start_address(addr: PhysAddr) -> Self {
        Self(addr)
    }

    pub fn addr(&self) -> u64 {
        self.0.as_u64()
    }

    pub fn start_address(&self) -> PhysAddr {
        self.0
    }
}
pub const FRAME_SIZE: u64 = 4096;

/// Instância global do alocador de frames (Protegido por Spinlock)
pub static FRAME_ALLOCATOR: Spinlock<BitmapFrameAllocator> =
    Spinlock::new(BitmapFrameAllocator::new());

pub fn init() {
    // TODO: Init PMM
}
