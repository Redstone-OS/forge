//! # Semaphore
//!
//! Semáforo de contagem para controle de recursos.
//!
//! ## Operações
//!
//! - `acquire` (P/wait/down): Decrementa, bloqueia se zero
//! - `release` (V/signal/up): Incrementa, acorda quem espera

use core::sync::atomic::{AtomicI32, Ordering};

/// Semáforo de contagem.
pub struct Semaphore {
    count: AtomicI32,
    max: i32,
}

impl Semaphore {
    /// Cria semáforo com contagem inicial.
    #[inline]
    pub const fn new(initial: i32) -> Self {
        Self {
            count: AtomicI32::new(initial),
            max: initial,
        }
    }

    /// Cria semáforo binário (mutex simples).
    #[inline]
    pub const fn binary() -> Self {
        Self::new(1)
    }

    /// Decrementa (P/wait/acquire).
    ///
    /// Bloqueia se contagem <= 0.
    #[inline]
    pub fn acquire(&self) {
        loop {
            let count = self.count.load(Ordering::Acquire);
            if count <= 0 {
                // TODO: Integrar com scheduler para dormir
                core::hint::spin_loop();
                continue;
            }

            if self
                .count
                .compare_exchange_weak(count, count - 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return;
            }
        }
    }

    /// Tenta decrementar sem bloquear.
    #[inline]
    pub fn try_acquire(&self) -> bool {
        loop {
            let count = self.count.load(Ordering::Acquire);
            if count <= 0 {
                return false;
            }

            if self
                .count
                .compare_exchange_weak(count, count - 1, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return true;
            }
        }
    }

    /// Incrementa (V/signal/release).
    #[inline]
    pub fn release(&self) {
        self.count.fetch_add(1, Ordering::Release);
        // TODO: Acordar threads esperando
    }

    /// Contagem atual.
    #[inline]
    pub fn count(&self) -> i32 {
        self.count.load(Ordering::Relaxed)
    }

    /// Contagem máxima (inicial).
    #[inline]
    pub fn max(&self) -> i32 {
        self.max
    }

    /// Recursos disponíveis.
    #[inline]
    pub fn available(&self) -> i32 {
        let c = self.count.load(Ordering::Relaxed);
        if c > 0 {
            c
        } else {
            0
        }
    }
}

impl Default for Semaphore {
    fn default() -> Self {
        Self::new(1)
    }
}
