//! # Read-Copy-Update (RCU)
//!
//! Sincronização otimizada para cenários com muitas leituras e poucas escritas.
//!
//! ## Características
//!
//! - Leitura: Lock-free, apenas incrementa refcount
//! - Escrita: Cria cópia, troca ponteiro atomicamente
//!
//! Ideal para configurações globais, listas de processos, etc.

use alloc::sync::Arc;
use core::ops::Deref;
use core::sync::atomic::{AtomicPtr, Ordering};

/// Container RCU para dados compartilhados.
pub struct Rcu<T> {
    inner: AtomicPtr<T>,
}

// SAFETY: RCU usa operações atômicas para sincronização
unsafe impl<T: Send + Sync> Send for Rcu<T> {}
unsafe impl<T: Send + Sync> Sync for Rcu<T> {}

impl<T> Rcu<T> {
    /// Cria novo container RCU.
    pub fn new(data: T) -> Self {
        let ptr = Arc::into_raw(Arc::new(data)) as *mut T;
        Self {
            inner: AtomicPtr::new(ptr),
        }
    }

    /// Leitura RCU (lock-free).
    ///
    /// Retorna guard que mantém referência aos dados.
    pub fn read(&self) -> RcuReadGuard<T> {
        let ptr = self.inner.load(Ordering::Acquire);

        // Incrementar refcount para manter dados vivos
        unsafe {
            Arc::increment_strong_count(ptr);
        }

        RcuReadGuard {
            ptr: unsafe { &*ptr },
            raw: ptr,
        }
    }

    /// Atualização RCU (writer).
    ///
    /// Cria nova versão e troca ponteiro atomicamente.
    /// Leitores antigos continuam usando versão antiga até terminarem.
    pub fn update(&self, new_data: T) {
        let new_ptr = Arc::into_raw(Arc::new(new_data)) as *mut T;

        // Troca atômica do ponteiro
        let old_ptr = self.inner.swap(new_ptr, Ordering::AcqRel);

        // Decrementar refcount do antigo
        // Se não houver mais leitores, será liberado
        unsafe {
            Arc::decrement_strong_count(old_ptr);
        }
    }

    /// Atualiza com função de transformação.
    pub fn update_with<F>(&self, f: F)
    where
        T: Clone,
        F: FnOnce(&T) -> T,
    {
        let current = self.read();
        let new_data = f(&*current);
        drop(current);
        self.update(new_data);
    }
}

impl<T> Drop for Rcu<T> {
    fn drop(&mut self) {
        let ptr = self.inner.load(Ordering::Acquire);
        unsafe {
            Arc::decrement_strong_count(ptr);
        }
    }
}

/// Guard de leitura RCU.
pub struct RcuReadGuard<'a, T> {
    ptr: &'a T,
    raw: *const T,
}

impl<T> Deref for RcuReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.ptr
    }
}

impl<T> Drop for RcuReadGuard<'_, T> {
    fn drop(&mut self) {
        // Decrementar refcount
        unsafe {
            Arc::decrement_strong_count(self.raw);
        }
    }
}

// Guards não podem ser enviados entre threads
impl<T> !Send for RcuReadGuard<'_, T> {}
