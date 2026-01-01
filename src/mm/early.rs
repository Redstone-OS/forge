//! # Early Boot Allocator
//!
//! Alocador simples para uso antes do heap estar disponível.

use crate::sync::Spinlock;
use core::alloc::Layout;

#[derive(Debug, Clone, Copy)]
pub struct EarlyRegion {
    pub start: u64,
    pub current: u64,
    pub end: u64,
}

impl EarlyRegion {
    pub const fn new(start: u64, end: u64) -> Self {
        Self {
            start,
            current: start,
            end,
        }
    }
    pub fn available(&self) -> u64 {
        self.end.saturating_sub(self.current)
    }
}

struct EarlyAllocatorState {
    region: Option<EarlyRegion>,
    total_allocated: u64,
}

impl EarlyAllocatorState {
    const fn new() -> Self {
        Self {
            region: None,
            total_allocated: 0,
        }
    }
}

static EARLY_ALLOCATOR: Spinlock<EarlyAllocatorState> = Spinlock::new(EarlyAllocatorState::new());

/// Inicializa o early allocator
pub unsafe fn init(start: u64, end: u64) {
    let mut state = EARLY_ALLOCATOR.lock();
    state.region = Some(EarlyRegion::new(start, end));
    state.total_allocated = 0;
}

/// Aloca memória física
pub unsafe fn alloc_phys(size: usize, align: usize) -> Option<u64> {
    let mut state = EARLY_ALLOCATOR.lock();
    let region = state.region.as_mut()?;

    let aligned = crate::klib::align_up(region.current as usize, align) as u64;
    let end = aligned.checked_add(size as u64)?;

    if end > region.end {
        return None;
    }

    region.current = end;
    state.total_allocated += size as u64;
    Some(aligned)
}

/// Aloca frames físicos
pub unsafe fn alloc_frames(count: usize) -> Option<u64> {
    alloc_phys(
        count * crate::mm::config::PAGE_SIZE,
        crate::mm::config::PAGE_SIZE,
    )
}

/// Reserva memória para FrameInfo array
pub unsafe fn reserve_frame_info_array<T>(total_frames: usize) -> Option<*mut T> {
    let size = core::mem::size_of::<T>() * total_frames;
    let align = core::mem::align_of::<T>();
    let phys = alloc_phys(size, align)?;
    let ptr: *mut T = crate::mm::hhdm::phys_to_virt(phys);
    core::ptr::write_bytes(ptr as *mut u8, 0, size);
    Some(ptr)
}
