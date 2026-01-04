//! # RwLock
//!
//! Read-Write Lock: múltiplos leitores OU um escritor exclusivo.
//!
//! ## Estado Interno
//!
//! - `0`: Livre
//! - `N > 0`: N leitores ativos
//! - `-1`: Escritor ativo

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicI32, Ordering};

/// RwLock - múltiplos leitores OU um escritor.
pub struct RwLock<T> {
    state: AtomicI32,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    /// Cria novo RwLock.
    #[inline]
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicI32::new(0),
            data: UnsafeCell::new(data),
        }
    }

    /// Adquire lock de leitura (múltiplos permitidos).
    #[inline]
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        loop {
            let state = self.state.load(Ordering::Acquire);

            // Se escritor ativo, esperar
            if state < 0 {
                core::hint::spin_loop();
                continue;
            }

            // Tentar incrementar leitores
            if self
                .state
                .compare_exchange_weak(state, state + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return RwLockReadGuard { lock: self };
            }
        }
    }

    /// Tenta adquirir lock de leitura sem bloquear.
    #[inline]
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        let state = self.state.load(Ordering::Acquire);
        if state < 0 {
            return None;
        }

        if self
            .state
            .compare_exchange(state, state + 1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(RwLockReadGuard { lock: self })
        } else {
            None
        }
    }

    /// Adquire lock de escrita (exclusivo).
    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        loop {
            if self
                .state
                .compare_exchange_weak(0, -1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return RwLockWriteGuard { lock: self };
            }
            core::hint::spin_loop();
        }
    }

    /// Tenta adquirir lock de escrita sem bloquear.
    #[inline]
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        if self
            .state
            .compare_exchange(0, -1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(RwLockWriteGuard { lock: self })
        } else {
            None
        }
    }

    /// Número de leitores ativos (0 se escritor ativo).
    #[inline]
    pub fn reader_count(&self) -> u32 {
        let state = self.state.load(Ordering::Relaxed);
        if state > 0 {
            state as u32
        } else {
            0
        }
    }

    /// Verifica se tem escritor ativo.
    #[inline]
    pub fn is_write_locked(&self) -> bool {
        self.state.load(Ordering::Relaxed) < 0
    }
}

impl<T: Default> Default for RwLock<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Guard de leitura.
pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: Lock de leitura adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for RwLockReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

/// Guard de escrita.
pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: Lock de escrita adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: Lock de escrita adquirido
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for RwLockWriteGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
    }
}
