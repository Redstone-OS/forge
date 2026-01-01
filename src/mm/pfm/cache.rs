//! # Per-CPU Frame Cache
//!
//! Cache de frames por CPU para alocação rápida.

use crate::mm::PhysAddr;
use core::sync::atomic::{AtomicUsize, Ordering};

const CACHE_SIZE: usize = 32;
const MAX_CPUS: usize = 64;

pub struct CpuFrameCache {
    frames: [u64; CACHE_SIZE],
    count: usize,
    cpu_id: usize,
    hits: u64,
    misses: u64,
}

impl CpuFrameCache {
    pub fn new(cpu_id: usize) -> Self {
        Self {
            frames: [0; CACHE_SIZE],
            count: 0,
            cpu_id,
            hits: 0,
            misses: 0,
        }
    }

    pub fn alloc(&mut self) -> Option<PhysAddr> {
        if self.count > 0 {
            self.count -= 1;
            self.hits += 1;
            Some(PhysAddr::new(self.frames[self.count]))
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn free(&mut self, phys: PhysAddr) -> bool {
        if self.count < CACHE_SIZE {
            self.frames[self.count] = phys.as_u64();
            self.count += 1;
            true
        } else {
            false
        }
    }

    pub fn refill(&mut self, count: usize) {
        let pfm = super::get().lock();
        let to_fill = core::cmp::min(count, CACHE_SIZE - self.count);
        for _ in 0..to_fill {
            if let Ok(phys) = crate::mm::pfm::alloc_kernel_frame() {
                self.frames[self.count] = phys.as_u64();
                self.count += 1;
            } else {
                break;
            }
        }
    }

    pub fn drain(&mut self) -> usize {
        let drained = self.count;
        for i in 0..self.count {
            let phys = PhysAddr::new(self.frames[i]);
            let _ = crate::mm::pfm::free_frame(phys, super::PID_KERNEL);
        }
        self.count = 0;
        drained
    }
}

static CURRENT_CPU: AtomicUsize = AtomicUsize::new(0);

pub fn get_cpu_id() -> usize {
    CURRENT_CPU.load(Ordering::Relaxed)
}
