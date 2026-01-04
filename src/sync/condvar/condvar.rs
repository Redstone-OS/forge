//! # Condition Variable
//!
//! Permite que threads esperem por uma condição específica.
//!
//! ## Status
//!
//! Atualmente usa spin-wait. TODO: Integrar com scheduler.

use crate::sync::mutex::MutexGuard;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Condition Variable.
///
/// Permite esperar por uma condição, liberando mutex atomicamente.
pub struct CondVar {
    signal_counter: AtomicUsize,
    waiters: AtomicUsize,
}

impl CondVar {
    /// Cria nova condition variable.
    #[inline]
    pub const fn new() -> Self {
        Self {
            signal_counter: AtomicUsize::new(0),
            waiters: AtomicUsize::new(0),
        }
    }

    /// Espera pela condição.
    ///
    /// Libera o mutex atomicamente, dorme, e readquire ao acordar.
    pub fn wait<T>(&self, _guard: &mut MutexGuard<'_, T>) {
        let current_signal = self.signal_counter.load(Ordering::Relaxed);
        self.waiters.fetch_add(1, Ordering::Relaxed);

        // TODO: Implementação real:
        // 1. guard.unlock_and_sleep(self);
        // 2. scheduler.wake() chamado por notify
        // 3. guard.lock();

        // Placeholder: busy-wait
        loop {
            if self.signal_counter.load(Ordering::Acquire) != current_signal {
                break;
            }
            core::hint::spin_loop();
        }

        self.waiters.fetch_sub(1, Ordering::Relaxed);
    }

    /// Espera com timeout.
    ///
    /// Retorna `true` se foi sinalizado, `false` se timeout.
    pub fn wait_timeout<T>(&self, guard: &mut MutexGuard<'_, T>, _timeout_ns: u64) -> bool {
        // TODO: Implementar timeout real
        self.wait(guard);
        true
    }

    /// Acorda uma thread esperando.
    #[inline]
    pub fn notify_one(&self) {
        self.signal_counter.fetch_add(1, Ordering::Release);
        // TODO: scheduler.wake_one(self);
    }

    /// Acorda todas as threads esperando.
    #[inline]
    pub fn notify_all(&self) {
        self.signal_counter.fetch_add(1, Ordering::Release);
        // TODO: scheduler.wake_all(self);
    }

    /// Número de threads esperando.
    #[inline]
    pub fn waiters(&self) -> usize {
        self.waiters.load(Ordering::Relaxed)
    }

    /// Verifica se tem alguém esperando.
    #[inline]
    pub fn has_waiters(&self) -> bool {
        self.waiters() > 0
    }
}

impl Default for CondVar {
    fn default() -> Self {
        Self::new()
    }
}
