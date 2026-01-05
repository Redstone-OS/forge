//! # Estatísticas do Gerenciador Físico

use core::sync::atomic::{AtomicU64, Ordering};

/// Estatísticas globais
pub struct PhysStats {
    pub total_frames: AtomicU64,
    pub free_frames: AtomicU64,
    pub kernel_frames: AtomicU64,
    pub user_frames: AtomicU64,
    pub pinned_frames: AtomicU64,
    pub alloc_count: AtomicU64,
    pub free_count: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
}

impl PhysStats {
    pub const fn new() -> Self {
        Self {
            total_frames: AtomicU64::new(0),
            free_frames: AtomicU64::new(0),
            kernel_frames: AtomicU64::new(0),
            user_frames: AtomicU64::new(0),
            pinned_frames: AtomicU64::new(0),
            alloc_count: AtomicU64::new(0),
            free_count: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }

    pub fn record_alloc(&self, from_cache: bool) {
        self.alloc_count.fetch_add(1, Ordering::Relaxed);
        if from_cache {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_free(&self) {
        self.free_count.fetch_add(1, Ordering::Relaxed);
    }
}
