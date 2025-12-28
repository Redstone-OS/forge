//! Reader-Writer Lock

use core::sync::atomic::{AtomicI32, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// RwLock - múltiplos leitores OU um escritor
/// 
/// Contador:
/// - 0 = Livre
/// - N>0 = N leitores ativos
/// - -1 = Escritor ativo
pub struct RwLock<T> {
    state: AtomicI32,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicI32::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    /// Adquire lock de leitura
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        loop {
            let state = self.state.load(Ordering::Acquire);
            
            // Se escritor ativo, esperar
            if state < 0 {
                core::hint::spin_loop();
                continue;
            }
            
            // Tentar incrementar leitores
            if self.state.compare_exchange_weak(
                state,
                state + 1,
                Ordering::Acquire,
                Ordering::Relaxed
            ).is_ok() {
                return RwLockReadGuard { lock: self };
            }
        }
    }
    
    /// Adquire lock de escrita
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        loop {
            // Tentar adquirir se livre
            if self.state.compare_exchange_weak(
                0,
                -1,
                Ordering::Acquire,
                Ordering::Relaxed
            ).is_ok() {
                return RwLockWriteGuard { lock: self };
            }
            core::hint::spin_loop();
        }
    }
}

pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: Lock de leitura adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: Lock de escrita adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: Lock de escrita adquirido
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
    }
}
