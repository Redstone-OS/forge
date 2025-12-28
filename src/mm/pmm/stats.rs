use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Default)]
pub struct PmmStats {
    pub total_frames: usize,
    pub used_frames: AtomicUsize,
    pub failed_allocs: AtomicUsize,
}

impl PmmStats {
    pub const fn new() -> Self {
        Self {
            total_frames: 0,
            used_frames: AtomicUsize::new(0),
            failed_allocs: AtomicUsize::new(0),
        }
    }

    pub fn inc_alloc(&self) {
        self.used_frames.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_free(&self) {
        self.used_frames.fetch_sub(1, Ordering::Relaxed);
    }
}
