//! # Mutex
//!
//! Bloqueio que pode colocar thread para dormir.
//!
//! ## Diferença do Spinlock
//!
//! - Mutex PODE dormir (cede CPU ao scheduler)
//! - Spinlock NÃO pode dormir (busy-wait)
//!
//! Use Mutex para seções mais longas ou que envolvem I/O.
//!
//! ## Status
//!
//! Atualmente usa spin-wait. TODO: Integrar com wait queue do scheduler.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Mutex - bloqueio que pode dormir.
///
/// **PROIBIDO** usar em interrupt handlers!
pub struct Mutex<T> {
    locked: AtomicBool,
    owner: AtomicU32,
    data: UnsafeCell<T>,
}

// SAFETY: Mutex protege acesso com lock
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Cria novo mutex.
    #[inline]
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            owner: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }

    /// Adquire o lock (pode bloquear).
    ///
    /// Atualmente usa spin-wait. Futuramente dormirá.
    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        // Tentar adquirir
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // TODO: Integrar com scheduler para dormir
            // Por enquanto, spin
            core::hint::spin_loop();
        }

        // TODO: Registrar owner para debug
        // self.owner.store(current_task_id(), Ordering::Relaxed);

        MutexGuard { lock: self }
    }

    /// Tenta adquirir sem bloquear.
    #[inline]
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(MutexGuard { lock: self })
        } else {
            None
        }
    }

    /// Verifica se está travado.
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    /// Retorna ID do owner atual (0 se livre).
    #[inline]
    pub fn owner(&self) -> u32 {
        self.owner.load(Ordering::Relaxed)
    }
}

impl<T: Default> Default for Mutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Guard do mutex - libera ao sair do escopo.
pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: Lock está adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: Lock está adquirido
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.owner.store(0, Ordering::Release);
        self.lock.locked.store(false, Ordering::Release);
        // TODO: Acordar threads esperando
    }
}

// Não permite enviar guard entre threads
impl<T> !Send for MutexGuard<'_, T> {}
