//! Mutex - pode bloquear thread

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Mutex - bloqueia thread se não conseguir lock
/// 
/// # Diferença do Spinlock
/// 
/// - Mutex PODE dormir (chama scheduler)
/// - Spinlock NÃO pode dormir (busy-wait)
/// 
/// Use Mutex para seções mais longas.
pub struct Mutex<T> {
    /// Estado do lock
    locked: AtomicBool,
    /// ID do owner (para debug)
    owner: AtomicU32,
    /// Dados protegidos
    data: UnsafeCell<T>,
}

// SAFETY: Mutex protege acesso com lock
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            owner: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    /// Adquire o lock (pode bloquear)
    pub fn lock(&self) -> MutexGuard<'_, T> {
        // Tentar adquirir
        while self.locked.compare_exchange_weak(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            // TODO: Integrar com scheduler para dormir
            // Por enquanto, spin
            core::hint::spin_loop();
        }
        
        MutexGuard { lock: self }
    }
    
    /// Tenta adquirir sem bloquear
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.locked.compare_exchange(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(MutexGuard { lock: self })
        } else {
            None
        }
    }
}

pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        // SAFETY: Lock está adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: Lock está adquirido
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.owner.store(0, Ordering::Release);
        self.lock.locked.store(false, Ordering::Release);
        // TODO: Acordar threads esperando
    }
}
