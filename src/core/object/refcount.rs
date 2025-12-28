/// Arquivo: core/object/refcount.rs
///
/// Propósito: Contagem de referências atômica para Objetos de Kernel.
/// Base para o gerenciamento de ciclo de vida de objetos compartilhados (KObjects).
///
/// Detalhes de Implementação:
/// - Usa `AtomicUsize` para thread-safety.
/// - Implementa semântica de Acquire/Release para garantir visibilidade correta
///   ao decrementar a última referência.

//! Reference Counting

use core::sync::atomic::{AtomicUsize, Ordering};

/// Contador de referências atômico
#[derive(Debug)]
pub struct RefCount {
    count: AtomicUsize,
}

impl RefCount {
    /// Cria um novo contador com valor inicial
    pub const fn new(initial: usize) -> Self {
        Self {
            count: AtomicUsize::new(initial),
        }
    }

    /// Incrementa o contador de referências.
    /// Retorna o valor ANTERIOR.
    #[inline]
    pub fn inc(&self) -> usize {
        // Relaxed é suficiente para incremento se não houver lógica de "start from zero"
        // complexa. Para kernel objects, geralmente assumimos que quem chama inc()
        // já segura uma referência válida (refcount > 0).
        self.count.fetch_add(1, Ordering::Relaxed)
    }

    /// Decrementa o contador de referências.
    /// Retorna `true` se a contagem chegou a ZERO (o objeto deve ser destruído).
    #[inline]
    #[must_use]
    pub fn dec(&self) -> bool {
        // fetch_sub retorna o valor ANTERIOR.
        // Release garante que escritas anteriores a este dec sejam vistas
        // antes de qualquer coisa que aconteça após o objeto morrer.
        let prev = self.count.fetch_sub(1, Ordering::Release);
        
        if prev == 1 {
            // Acquire fence é necessária para garantir que, se este thread destruir o objeto,
            // ele veja todas as modificações feitas por outros threads que deram release.
            core::sync::atomic::fence(Ordering::Acquire);
            true
        } else {
            false
        }
    }

    /// Retorna o valor atual (aproximado/relaxado).
    #[inline]
    pub fn get(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}
