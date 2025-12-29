//! Condition Variable

use crate::sync::mutex::MutexGuard;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Condition Variable
/// Permite que threads esperem por uma condição específica.
pub struct CondVar {
    // TODO: Usar WaitQueue do scheduler
    // Por enquanto, usamos um contador de notificações para spin-waiting (ineficiente, mas funcional para boot)
    signal_counter: AtomicUsize,
}

impl CondVar {
    pub const fn new() -> Self {
        Self {
            signal_counter: AtomicUsize::new(0),
        }
    }

    /// Espera pela condição.
    /// Libera o lock atômica e dorme (simulado com spin) até ser notificado.
    pub fn wait<T>(&self, _guard: &mut MutexGuard<'_, T>) {
        let current_signal = self.signal_counter.load(Ordering::Relaxed);

        // 1. Liberar Mutex (Drop manual simulado ou suporte no Mutex)
        // Como MutexGuard não tem unlock_and_sleep, vamos apenas...
        // Essa implementação REQUER suporte do Scheduler para ser correta e eficiente.
        // Implementação dummy apenas para compilação e interfaces.

        // TODO: guard.lock.unlock();
        // TODO: scheduler.wait(self);
        // TODO: guard.lock.lock();

        // Placeholder busy-wait:
        loop {
            if self.signal_counter.load(Ordering::Relaxed) != current_signal {
                break;
            }
            core::hint::spin_loop();
        }
    }

    /// Acorda uma thread esperando.
    pub fn notify_one(&self) {
        self.signal_counter.fetch_add(1, Ordering::Relaxed);
        // TODO: scheduler.wake_one(self);
    }

    /// Acorda todas as threads esperando.
    pub fn notify_all(&self) {
        self.signal_counter.fetch_add(1, Ordering::Relaxed);
        // TODO: scheduler.wake_all(self);
    }
}
