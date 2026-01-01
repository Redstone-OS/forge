//! # IOMMU API
//!
//! Gerenciamento de frames para DMA.

use super::{frame::FrameFlags, PfmResult, Pid};
use crate::mm::PhysAddr;

pub struct DmaRegion {
    pub phys_start: PhysAddr,
    pub size: usize,
    pub owner: Pid,
    pub device_id: u32,
}

pub fn alloc_dma_region(size: usize, owner: Pid, device_id: u32) -> PfmResult<DmaRegion> {
    let pages = (size + crate::mm::config::PAGE_SIZE - 1) / crate::mm::config::PAGE_SIZE;
    let phys = super::get()
        .lock()
        .alloc_contiguous(owner, pages, FrameFlags::empty())?;

    let mut pfm = super::get().lock();
    for i in 0..pages {
        let addr = PhysAddr::new(phys.as_u64() + (i * crate::mm::config::PAGE_SIZE) as u64);
        let _ = pfm.pin_frame(addr, owner);
        let _ = pfm.mark_device(addr);
    }

    Ok(DmaRegion {
        phys_start: phys,
        size,
        owner,
        device_id,
    })
}

pub fn free_dma_region(region: &DmaRegion) -> PfmResult<()> {
    let pages = (region.size + crate::mm::config::PAGE_SIZE - 1) / crate::mm::config::PAGE_SIZE;
    let mut pfm = super::get().lock();

    for i in 0..pages {
        let addr =
            PhysAddr::new(region.phys_start.as_u64() + (i * crate::mm::config::PAGE_SIZE) as u64);
        let _ = pfm.unpin_frame(addr, region.owner);
        let _ = pfm.free_frame(addr, region.owner);
    }
    Ok(())
}
