//! # Spinlock
//!
//! Bloqueio com busy-wait e desabilitação de interrupções.
//!
//! ## Quando Usar
//!
//! - Seções críticas MUITO curtas (< 1µs)
//! - Dentro de handlers de interrupção
//! - Quando não pode chamar scheduler
//!
//! ## Quando NÃO Usar
//!
//! - Seções que podem demorar
//! - Quando pode chamar funções que dormem
//! - Para proteger I/O lento

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// Spinlock - bloqueio com busy-wait, interrupt-safe.
///
/// Desabilita interrupções automaticamente para evitar deadlock
/// quando IRQ handler tenta adquirir o mesmo lock.
pub struct Spinlock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: Spinlock protege acesso com lock atômico
unsafe impl<T: Send> Send for Spinlock<T> {}
unsafe impl<T: Send> Sync for Spinlock<T> {}

impl<T> Spinlock<T> {
    /// Cria novo spinlock.
    #[inline]
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Adquire o lock (bloqueia até conseguir).
    ///
    /// Desabilita interrupções enquanto o lock está ativo.
    #[inline]
    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        // Salvar e desabilitar interrupções
        let interrupts_enabled = crate::arch::Cpu::interrupts_enabled();
        crate::arch::Cpu::disable_interrupts();

        // Spin até conseguir o lock
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // Hint para CPU otimizar spin loop
            core::hint::spin_loop();
        }

        SpinlockGuard {
            lock: self,
            interrupts_were_enabled: interrupts_enabled,
        }
    }

    /// Tenta adquirir sem bloquear.
    ///
    /// Retorna `None` se já está travado.
    #[inline]
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

    /// Verifica se está travado (sem adquirir).
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    /// Força desbloqueio (APENAS USO INTERNO DO SCHEDULER).
    ///
    /// # Safety
    ///
    /// Extremamente inseguro. Só deve ser usado pelo scheduler
    /// ao iniciar nova task que "herdou" lock da anterior.
    #[inline]
    pub unsafe fn force_unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }

    /// Acesso aos dados sem lock (APENAS PARA INICIALIZAÇÃO).
    ///
    /// # Safety
    ///
    /// Só usar quando há garantia de acesso único (boot, single-threaded).
    #[inline]
    pub unsafe fn get_mut_unchecked(&self) -> &mut T {
        &mut *self.data.get()
    }
}

impl<T: Default> Default for Spinlock<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Guard do spinlock - libera ao sair do escopo.
pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
    interrupts_were_enabled: bool,
}

impl<T> Deref for SpinlockGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: Lock está adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinlockGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: Lock está adquirido
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinlockGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Liberar lock
        self.lock.locked.store(false, Ordering::Release);

        // Restaurar interrupções
        if self.interrupts_were_enabled {
            crate::arch::Cpu::enable_interrupts();
        }
    }
}

// Não permite enviar guard entre threads (contém referência ao lock)
impl<T> !Send for SpinlockGuard<'_, T> {}
