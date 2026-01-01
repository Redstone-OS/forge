//! # Memory Statistics

use core::sync::atomic::{AtomicU64, Ordering};

pub static TOTAL_PHYSICAL: AtomicU64 = AtomicU64::new(0);
pub static FREE_PHYSICAL: AtomicU64 = AtomicU64::new(0);
pub static KERNEL_USAGE: AtomicU64 = AtomicU64::new(0);
pub static USER_USAGE: AtomicU64 = AtomicU64::new(0);
pub static SHARED_PAGES: AtomicU64 = AtomicU64::new(0);
pub static DIRTY_PAGES: AtomicU64 = AtomicU64::new(0);
pub static PAGE_FAULTS: AtomicU64 = AtomicU64::new(0);
pub static COW_FAULTS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub total_physical: u64,
    pub free_physical: u64,
    pub kernel_usage: u64,
    pub user_usage: u64,
    pub shared_pages: u64,
    pub dirty_pages: u64,
    pub page_faults: u64,
    pub cow_faults: u64,
}

impl MemoryStats {
    pub fn usage_percent(&self) -> u64 {
        if self.total_physical == 0 {
            return 0;
        }
        let used = self.total_physical - self.free_physical;
        (used * 100) / self.total_physical
    }
}

pub fn snapshot() -> MemoryStats {
    MemoryStats {
        total_physical: TOTAL_PHYSICAL.load(Ordering::Relaxed),
        free_physical: FREE_PHYSICAL.load(Ordering::Relaxed),
        kernel_usage: KERNEL_USAGE.load(Ordering::Relaxed),
        user_usage: USER_USAGE.load(Ordering::Relaxed),
        shared_pages: SHARED_PAGES.load(Ordering::Relaxed),
        dirty_pages: DIRTY_PAGES.load(Ordering::Relaxed),
        page_faults: PAGE_FAULTS.load(Ordering::Relaxed),
        cow_faults: COW_FAULTS.load(Ordering::Relaxed),
    }
}

pub fn update_from_pfm() {
    if let Some(pfm) = crate::mm::pfm::get().try_lock() {
        let stats = pfm.stats();
        TOTAL_PHYSICAL.store(stats.total_frames * 4, Ordering::Relaxed);
        FREE_PHYSICAL.store(stats.free_frames * 4, Ordering::Relaxed);
        KERNEL_USAGE.store(stats.kernel_frames * 4, Ordering::Relaxed);
        USER_USAGE.store(stats.user_frames * 4, Ordering::Relaxed);
        SHARED_PAGES.store(stats.shared_frames, Ordering::Relaxed);
    }
}
