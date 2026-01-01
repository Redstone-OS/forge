//! # Shared Memory

use super::{ASpaceError, ASpaceResult, Pid};
use crate::mm::{PhysAddr, VirtAddr};
use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedMemHandle(pub u64);

pub struct SharedMemRegion {
    pub handle: SharedMemHandle,
    pub phys_base: PhysAddr,
    pub size: usize,
    pub ref_count: u32,
    pub creator: Pid,
}

impl SharedMemRegion {
    pub fn new(handle: SharedMemHandle, phys_base: PhysAddr, size: usize, creator: Pid) -> Self {
        Self {
            handle,
            phys_base,
            size,
            ref_count: 1,
            creator,
        }
    }
    pub fn pages(&self) -> usize {
        (self.size + crate::mm::config::PAGE_SIZE - 1) / crate::mm::config::PAGE_SIZE
    }
}

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

pub fn create(size: usize, creator: Pid) -> ASpaceResult<SharedMemRegion> {
    let pages = (size + crate::mm::config::PAGE_SIZE - 1) / crate::mm::config::PAGE_SIZE;
    let mut allocator = crate::mm::pmm::FRAME_ALLOCATOR.lock();
    let phys_base = allocator.allocate_frame().ok_or(ASpaceError::OutOfMemory)?;
    for _ in 1..pages {
        allocator.allocate_frame().ok_or(ASpaceError::OutOfMemory)?;
    }
    let handle = SharedMemHandle(NEXT_HANDLE.fetch_add(1, Ordering::SeqCst));
    Ok(SharedMemRegion::new(handle, phys_base, size, creator))
}

pub fn map_in(
    _region: &SharedMemRegion,
    target_addr: VirtAddr,
    _pid: Pid,
) -> ASpaceResult<VirtAddr> {
    Ok(target_addr)
}

pub fn unmap_from(region: &mut SharedMemRegion, _pid: Pid) -> ASpaceResult<()> {
    region.ref_count = region.ref_count.saturating_sub(1);
    Ok(())
}
