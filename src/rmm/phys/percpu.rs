//! # Per-CPU Cache
//!
//! Cache de frames por CPU para fast-path sem lock.

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PERCPU_CACHE_SIZE;

/// Cache local da CPU
pub struct PerCpuCache {
    /// Frames em cache
    frames: [PhysAddr; PERCPU_CACHE_SIZE],
    /// Quantidade atual
    count: usize,
}

impl PerCpuCache {
    pub const fn new() -> Self {
        Self {
            frames: [PhysAddr::new(0); PERCPU_CACHE_SIZE],
            count: 0,
        }
    }

    /// Remove frame do cache (O(1))
    pub fn pop(&mut self) -> Option<PhysAddr> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        Some(self.frames[self.count])
    }

    /// Adiciona frame ao cache (O(1))
    pub fn push(&mut self, frame: PhysAddr) -> bool {
        if self.count >= PERCPU_CACHE_SIZE {
            return false;
        }
        self.frames[self.count] = frame;
        self.count += 1;
        true
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn is_full(&self) -> bool {
        self.count >= PERCPU_CACHE_SIZE
    }

    pub fn len(&self) -> usize {
        self.count
    }
}

/// Array de caches per-CPU
pub struct PerCpuCaches {
    caches: [PerCpuCache; 256],
}

impl PerCpuCaches {
    pub const fn new() -> Self {
        const INIT: PerCpuCache = PerCpuCache::new();
        Self {
            caches: [INIT; 256],
        }
    }

    pub fn get(&mut self, cpu: usize) -> &mut PerCpuCache {
        &mut self.caches[cpu % 256]
    }
}
