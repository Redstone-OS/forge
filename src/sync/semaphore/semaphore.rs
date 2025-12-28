//! Semáforo para controle de recursos

use core::sync::atomic::{AtomicI32, Ordering};

/// Semáforo de contagem
pub struct Semaphore {
    count: AtomicI32,
}

impl Semaphore {
    pub const fn new(initial: i32) -> Self {
        Self {
            count: AtomicI32::new(initial),
        }
    }
    
    /// Decrementa (P/wait/acquire)
    pub fn acquire(&self) {
        loop {
            let count = self.count.load(Ordering::Acquire);
            if count <= 0 {
                // Esperar
                core::hint::spin_loop();
                continue;
            }
            
            if self.count.compare_exchange_weak(
                count,
                count - 1,
                Ordering::AcqRel,
                Ordering::Relaxed
            ).is_ok() {
                return;
            }
        }
    }
    
    /// Tenta decrementar sem bloquear
    pub fn try_acquire(&self) -> bool {
        let count = self.count.load(Ordering::Acquire);
        if count <= 0 {
            return false;
        }
        
        self.count.compare_exchange(
            count,
            count - 1,
            Ordering::AcqRel,
            Ordering::Relaxed
        ).is_ok()
    }
    
    /// Incrementa (V/signal/release)
    pub fn release(&self) {
        self.count.fetch_add(1, Ordering::Release);
    }
}
