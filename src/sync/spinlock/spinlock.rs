//! Spinlock - bloqueio com busy-wait

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// Spinlock - usa busy-wait, NÃO pode dormir
///
/// # Quando usar
///
/// - Seções críticas MUITO curtas
/// - Dentro de handlers de interrupção
/// - Quando não pode chamar scheduler
///
/// # Quando NÃO usar
///
/// - Seções que podem demorar
/// - Quando pode chamar funções que dormem
/// - Para proteger I/O lento
pub struct Spinlock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: Spinlock protege acesso com lock atômico
unsafe impl<T: Send> Send for Spinlock<T> {}
unsafe impl<T: Send> Sync for Spinlock<T> {}

impl<T> Spinlock<T> {
    /// Cria novo spinlock
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Adquire o lock
    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        // Desabilitar interrupções antes de adquirir
        let interrupts_enabled = crate::arch::Cpu::interrupts_enabled();
        crate::arch::Cpu::disable_interrupts();

        // Spin até conseguir o lock
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // Hint para CPU que estamos em spin loop
            core::hint::spin_loop();
        }

        SpinlockGuard {
            lock: self,
            interrupts_were_enabled: interrupts_enabled,
        }
    }

    /// Tenta adquirir sem bloquear
    pub fn try_lock(&self) -> Option<SpinlockGuard<'_, T>> {
        let interrupts_enabled = crate::arch::Cpu::interrupts_enabled();
        crate::arch::Cpu::disable_interrupts();

        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinlockGuard {
                lock: self,
                interrupts_were_enabled: interrupts_enabled,
            })
        } else {
            // Não conseguiu, restaurar interrupções
            if interrupts_enabled {
                crate::arch::Cpu::enable_interrupts();
            }
            None
        }
    }

    /// Força o desbloqueio do spinlock (USO INTERNO DO SCHEDULER)
    ///
    /// # Safety
    ///
    /// Extremamente inseguro. Só deve ser usado pelo scheduler ao iniciar
    /// uma nova task que "herdou" o lock da task anterior mas não tem o Guard.
    pub unsafe fn force_unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

/// Guard do spinlock - libera ao sair do escopo
pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
    interrupts_were_enabled: bool,
}

impl<T> Deref for SpinlockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: Lock está adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: Lock está adquirido
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinlockGuard<'_, T> {
    fn drop(&mut self) {
        // Liberar lock
        self.lock.locked.store(false, Ordering::Release);

        // Restaurar interrupções se estavam habilitadas
        if self.interrupts_were_enabled {
            crate::arch::Cpu::enable_interrupts();
        }
    }
}
