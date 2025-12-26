use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Estatísticas do Gerenciador de Memória Física
pub struct PmmStats {
    pub total_frames: usize,
    pub used_frames: AtomicUsize,
    pub allocated_total: AtomicU64,
    pub freed_total: AtomicU64,
}

impl PmmStats {
    pub const fn new() -> Self {
        Self {
            total_frames: 0,
            used_frames: AtomicUsize::new(0),
            allocated_total: AtomicU64::new(0),
            freed_total: AtomicU64::new(0),
        }
    }

    pub fn inc_alloc(&self) {
        self.used_frames.fetch_add(1, Ordering::Relaxed);
        self.allocated_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_free(&self) {
        self.used_frames.fetch_sub(1, Ordering::Relaxed);
        self.freed_total.fetch_add(1, Ordering::Relaxed);
    }
}
