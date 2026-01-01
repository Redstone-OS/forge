//! # Page Reclaim Subsystem

pub mod aging;
pub mod evict;
pub mod kswapd;
pub mod oom;

pub use aging::PageAger;
pub use evict::evict_pages;
pub use kswapd::start_kswapd;
pub use oom::oom_kill;

pub struct MemoryWatermarks {
    pub low: u64,
    pub high: u64,
    pub min: u64,
}

impl Default for MemoryWatermarks {
    fn default() -> Self {
        Self {
            low: 1024,
            high: 4096,
            min: 256,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressure {
    None,
    Low,
    Medium,
    Critical,
}

pub fn get_pressure() -> MemoryPressure {
    let pfm = crate::mm::pfm::get().lock();
    let stats = pfm.stats();
    let free = stats.free_frames;
    let wm = MemoryWatermarks::default();

    if free > wm.high {
        MemoryPressure::None
    } else if free > wm.low {
        MemoryPressure::Low
    } else if free > wm.min {
        MemoryPressure::Medium
    } else {
        MemoryPressure::Critical
    }
}
