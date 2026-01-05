//! # Shrinkers
//!
//! Interface para componentes do kernel liberarem memória sob demanda.
//!
//! ## Conceito
//!
//! Shrinkers são callbacks que o reclaimer chama quando precisa de memória.
//! Cada subsistema que usa caches pode registrar um shrinker.
//!
//! ## Exemplos
//!
//! - **Slab Cache**: Libera objetos não usados
//! - **Dentry Cache**: Libera entradas de diretório
//! - **Inode Cache**: Libera inodes em memória
//! - **Page Cache**: Libera páginas de arquivos
//!
//! ## Registro
//!
//! ```rust
//! struct MyShrinker;
//!
//! impl ShrinkerOps for MyShrinker {
//!     fn name(&self) -> &'static str { "my_cache" }
//!     fn count(&self) -> usize { 100 }
//!     fn shrink(&self, target: usize) -> usize {
//!         // Libera até 'target' objetos
//!         let freed = do_shrink(target);
//!         freed
//!     }
//! }
//!
//! reclaim::register_shrinker(&MY_SHRINKER);
//! ```

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

// =============================================================================
// ShrinkerOps Trait
// =============================================================================

/// Trait para shrinkers
pub trait ShrinkerOps: Send + Sync {
    /// Nome do shrinker (para debug)
    fn name(&self) -> &'static str;

    /// Número de objetos que podem ser liberados
    fn count(&self) -> usize;

    /// Libera até `target` objetos
    ///
    /// Retorna número de objetos realmente liberados.
    fn shrink(&self, target: usize) -> usize;

    /// Prioridade (maior = chamado primeiro)
    fn priority(&self) -> u32 {
        50 // Default: média
    }

    /// Pode liberar objetos atualmente?
    fn can_shrink(&self) -> bool {
        self.count() > 0
    }
}

// =============================================================================
// Shrinker
// =============================================================================

/// Wrapper para shrinker com estatísticas
pub struct Shrinker<T: ShrinkerOps> {
    /// Implementação
    inner: T,
    /// Total de objetos liberados
    total_freed: AtomicU64,
    /// Número de chamadas
    shrink_calls: AtomicU64,
    /// Objetos em uso na última contagem
    last_count: AtomicUsize,
}

impl<T: ShrinkerOps> Shrinker<T> {
    /// Cria novo shrinker
    pub const fn new(inner: T) -> Self {
        Self {
            inner,
            total_freed: AtomicU64::new(0),
            shrink_calls: AtomicU64::new(0),
            last_count: AtomicUsize::new(0),
        }
    }

    /// Obtém referência ao inner
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Estatísticas
    pub fn stats(&self) -> ShrinkerStats {
        ShrinkerStats {
            name: self.inner.name(),
            count: self.inner.count(),
            total_freed: self.total_freed.load(Ordering::Relaxed),
            shrink_calls: self.shrink_calls.load(Ordering::Relaxed),
        }
    }
}

impl<T: ShrinkerOps> ShrinkerOps for Shrinker<T> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn count(&self) -> usize {
        let count = self.inner.count();
        self.last_count.store(count, Ordering::Relaxed);
        count
    }

    fn shrink(&self, target: usize) -> usize {
        self.shrink_calls.fetch_add(1, Ordering::Relaxed);
        let freed = self.inner.shrink(target);
        self.total_freed.fetch_add(freed as u64, Ordering::Relaxed);
        freed
    }

    fn priority(&self) -> u32 {
        self.inner.priority()
    }
}

// =============================================================================
// ShrinkerStats
// =============================================================================

/// Estatísticas de um shrinker
#[derive(Debug, Clone)]
pub struct ShrinkerStats {
    pub name: &'static str,
    pub count: usize,
    pub total_freed: u64,
    pub shrink_calls: u64,
}

// =============================================================================
// Built-in Shrinkers
// =============================================================================

/// Shrinker para slab allocator
pub struct SlabShrinker {
    /// Referência ao slab (placeholder)
    _marker: core::marker::PhantomData<()>,
}

impl SlabShrinker {
    pub const fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl ShrinkerOps for SlabShrinker {
    fn name(&self) -> &'static str {
        "slab"
    }

    fn count(&self) -> usize {
        // TODO: Obter de heap::slab
        0
    }

    fn shrink(&self, _target: usize) -> usize {
        // TODO: Implementar shrink de slab
        0
    }

    fn priority(&self) -> u32 {
        30 // Baixa prioridade (tenta outros primeiro)
    }
}

/// Shrinker para page cache
pub struct PageCacheShrinker;

impl ShrinkerOps for PageCacheShrinker {
    fn name(&self) -> &'static str {
        "page_cache"
    }

    fn count(&self) -> usize {
        // TODO: Obter de cache::page_cache
        0
    }

    fn shrink(&self, _target: usize) -> usize {
        // TODO: Implementar shrink de page cache
        0
    }

    fn priority(&self) -> u32 {
        70 // Alta prioridade (página cache é fácil de descartar)
    }
}

/// Shrinker para dentry cache
pub struct DentryShrinker;

impl ShrinkerOps for DentryShrinker {
    fn name(&self) -> &'static str {
        "dentry"
    }

    fn count(&self) -> usize {
        // TODO: Obter de fs::dentry_cache
        0
    }

    fn shrink(&self, _target: usize) -> usize {
        // TODO: Implementar shrink de dentry
        0
    }

    fn priority(&self) -> u32 {
        60
    }
}

/// Shrinker para inode cache
pub struct InodeShrinker;

impl ShrinkerOps for InodeShrinker {
    fn name(&self) -> &'static str {
        "inode"
    }

    fn count(&self) -> usize {
        // TODO: Obter de fs::inode_cache
        0
    }

    fn shrink(&self, _target: usize) -> usize {
        // TODO: Implementar shrink de inode
        0
    }

    fn priority(&self) -> u32 {
        55
    }
}

// =============================================================================
// Shrinker Registry
// =============================================================================

/// Callback registry para shrinkers
pub struct ShrinkerRegistry {
    /// Lista de shrinkers ordenada por prioridade
    shrinkers: alloc::vec::Vec<&'static dyn ShrinkerOps>,
}

impl ShrinkerRegistry {
    /// Cria registry vazio
    pub const fn new() -> Self {
        Self {
            shrinkers: alloc::vec::Vec::new(),
        }
    }

    /// Registra shrinker
    pub fn register(&mut self, shrinker: &'static dyn ShrinkerOps) {
        self.shrinkers.push(shrinker);
        // Reordena por prioridade (maior primeiro)
        self.shrinkers
            .sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Remove shrinker pelo nome
    pub fn unregister(&mut self, name: &str) {
        self.shrinkers.retain(|s| s.name() != name);
    }

    /// Shrink todos os shrinkers
    ///
    /// Retorna total de objetos liberados.
    pub fn shrink_all(&self, target: usize) -> usize {
        let mut total_freed = 0;
        let mut remaining_target = target;

        for shrinker in &self.shrinkers {
            if remaining_target == 0 {
                break;
            }

            if shrinker.can_shrink() {
                // Calcula quanto pedir a este shrinker
                let available = shrinker.count();
                let ask = remaining_target.min(available);

                if ask > 0 {
                    let freed = shrinker.shrink(ask);
                    total_freed += freed;
                    remaining_target = remaining_target.saturating_sub(freed);
                }
            }
        }

        total_freed
    }

    /// Total de objetos que podem ser liberados
    pub fn total_count(&self) -> usize {
        self.shrinkers.iter().map(|s| s.count()).sum()
    }

    /// Número de shrinkers registrados
    pub fn len(&self) -> usize {
        self.shrinkers.len()
    }

    /// Está vazio?
    pub fn is_empty(&self) -> bool {
        self.shrinkers.is_empty()
    }

    /// Itera sobre shrinkers
    pub fn iter(&self) -> impl Iterator<Item = &&'static dyn ShrinkerOps> {
        self.shrinkers.iter()
    }
}

impl Default for ShrinkerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
