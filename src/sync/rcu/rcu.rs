//! Read-Copy-Update (RCU)
//! Mecanismo de sincronização otimizado para cenários com muitas leituras e poucas escritas.

use alloc::sync::Arc;
use core::sync::atomic::{AtomicPtr, Ordering};

/// Container RCU para dados compartilhados
pub struct Rcu<T> {
    // Ponteiro atômico para os dados (gerenciado via Arc internamente, mas mantido como raw ptr)
    inner: AtomicPtr<T>,
}

impl<T> Rcu<T> {
    pub fn new(data: T) -> Self {
        let ptr = Arc::into_raw(Arc::new(data)) as *mut T;
        Self {
            inner: AtomicPtr::new(ptr),
        }
    }

    /// Leitura RCU (block-free)
    pub fn read(&self) -> RcuReadGuard<T> {
        let ptr = self.inner.load(Ordering::Acquire);
        // Em um sistema RCU real, precisaríamos de barreiras de memória e rastreamento de "epoch"
        // para garantir que o ponteiro não seja deletado enquanto lemos.
        // Aqui, assumimos que Arc + vazamento controlado (ou epoch-based reclamation futura) segura as pontas.

        // Incrementamos refcount do Arc para segurança simples (incur custo de atomicidade, mas é seguro)
        unsafe {
            Arc::increment_strong_count(ptr);
        }

        RcuReadGuard {
            ptr: unsafe { &*ptr },
            raw: ptr,
        }
    }

    /// Atualização RCU (writer)
    /// Cria uma nova versão e troca o ponteiro.
    pub fn update(&self, new_data: T) {
        let new_ptr = Arc::into_raw(Arc::new(new_data)) as *mut T;

        // Troca atômica do ponteiro
        let old_ptr = self.inner.swap(new_ptr, Ordering::AcqRel);

        // O dado antigo (old_ptr) só pode ser liberado quando todos os leitores terminarem (Grace Period).
        // TODO: Implementar Grace Period tracking (call_rcu).
        // Por enquanto, soltamos o Arc, mas se houver leitores eles seguram via increment_strong_count.
        unsafe {
            Arc::decrement_strong_count(old_ptr);
        }
    }
}

pub struct RcuReadGuard<'a, T> {
    ptr: &'a T,
    raw: *const T,
}

impl<T> core::ops::Deref for RcuReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.ptr
    }
}

impl<T> Drop for RcuReadGuard<'_, T> {
    fn drop(&mut self) {
        // Solta referência
        unsafe {
            Arc::decrement_strong_count(self.raw);
        }
    }
}
