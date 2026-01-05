//! # Page Cache
//!
//! Cache de páginas de arquivos em RAM.
//!
//! ## Conceito
//!
//! O page cache mantém páginas de arquivos em memória para evitar I/O repetido.
//! Quando um arquivo é lido, as páginas ficam em cache. Leituras subsequentes
//! usam a memória ao invés de ir ao disco.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                        Page Cache                          │
//! ├────────────────────────────────────────────────────────────┤
//! │                                                            │
//! │  ┌──────────────────────────────────────────────────────┐  │
//! │  │                   PageIndex                          │  │
//! │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐     │  │
//! │  │  │ Inode 1 │ │ Inode 2 │ │ Inode 3 │ │   ...   │     │  │
//! │  │  │ pages[] │ │ pages[] │ │ pages[] │ │         │     │  │
//! │  │  └────┬────┘ └────┬────┘ └────┬────┘ └─────────┘     │  │
//! │  │       │           │           │                      │  │
//! │  └───────┼───────────┼───────────┼──────────────────────┘  │
//! │          │           │           │                         │
//! │          ▼           ▼           ▼                         │
//! │  ┌──────────────────────────────────────────────────────┐  │
//! │  │              Physical Frames (RAM)                   │  │
//! │  │           owner: FrameOwner::Cache { inode }         │  │
//! │  └──────────────────────────────────────────────────────┘  │
//! │                                                            │
//! │  ┌──────────────────────────────────────────────────────┐  │
//! │  │                 WritebackControl                     │  │
//! │  │              (dirty page sync)                       │  │
//! │  └──────────────────────────────────────────────────────┘  │
//! │                                                            │
//! │  ┌──────────────────────────────────────────────────────┐  │
//! │  │              PageCacheShrinker                       │  │
//! │  │           (reclaim integration)                      │  │
//! │  └──────────────────────────────────────────────────────┘  │
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Integração
//!
//! - **VFS**: Usa cache para read/write de arquivos
//! - **Reclaim**: Shrinker libera páginas sob pressão
//! - **Writeback**: Páginas sujas são escritas periodicamente
//!
//! ## API
//!
//! ```rust,ignore
//! // Busca página
//! if let Some(phys) = cache::lookup(inode, offset) {
//!     // Use cached page
//! }
//!
//! // Insere página
//! cache::insert(inode, offset, phys);
//!
//! // Invalida (após truncate/delete)
//! cache::invalidate(inode);
//!
//! // Writeback (antes de close/sync)
//! cache::writeback(inode);
//! ```

pub mod entry;
pub mod radix;
pub mod writeback;

pub use entry::{CacheEntry, CacheFlags, CacheKey};
pub use radix::{InodePages, PageIndex};
pub use writeback::{WritebackControl, WritebackRequest, WritebackStats};

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::reclaim::ShrinkerOps;
use crate::sync::Spinlock;

use core::sync::atomic::{AtomicU64, Ordering};

// =============================================================================
// Estado Global
// =============================================================================

/// Page cache global - usamos Option para inicialização lazy
static PAGE_CACHE: Spinlock<Option<PageCacheState>> = Spinlock::new(None);

/// Estatísticas atômicas
static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
static PAGES_ADDED: AtomicU64 = AtomicU64::new(0);
static PAGES_EVICTED: AtomicU64 = AtomicU64::new(0);

// =============================================================================
// PageCacheState
// =============================================================================

/// Estado interno do page cache
struct PageCacheState {
    /// Índice de páginas
    index: PageIndex,
    /// Controle de writeback
    writeback: WritebackControl,
    /// Limite máximo de páginas (0 = sem limite)
    max_pages: usize,
}

impl PageCacheState {
    fn new() -> Self {
        Self {
            index: PageIndex::new(),
            writeback: WritebackControl::new(),
            max_pages: 0, // Sem limite por default
        }
    }
}

// =============================================================================
// API Pública
// =============================================================================

/// Inicializa o page cache
pub fn init() {
    crate::kinfo!("(RMM/Cache) Inicializando page cache...");

    // Configura limite baseado na memória disponível
    // TODO: Obter memória total do phys manager
    let total_pages = 256 * 1024; // Placeholder: 1GB em páginas
    let max_cache = total_pages / 4; // 25% da RAM para cache

    {
        let mut cache = PAGE_CACHE.lock();
        let state = cache.get_or_insert_with(PageCacheState::new);
        state.max_pages = max_cache;
    }

    crate::kinfo!(
        "(RMM/Cache) Limite: {} páginas ({} MB)",
        max_cache,
        (max_cache * PAGE_SIZE) / (1024 * 1024)
    );
}

/// Busca página no cache
///
/// Retorna endereço físico se encontrada.
pub fn lookup(inode: u64, offset: u64) -> Option<PhysAddr> {
    // Alinha offset a página
    let aligned_offset = offset & !(PAGE_SIZE as u64 - 1);

    let cache = PAGE_CACHE.lock();
    let state = cache.as_ref()?;

    if let Some(entry) = state.index.get(inode, aligned_offset) {
        // Marca como referenciada
        entry.mark_referenced();
        CACHE_HITS.fetch_add(1, Ordering::Relaxed);
        Some(entry.phys())
    } else {
        CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
        None
    }
}

/// Insere página no cache
///
/// Se já existir página para (inode, offset), substitui.
pub fn insert(inode: u64, offset: u64, phys: PhysAddr) {
    // Alinha offset a página
    let aligned_offset = offset & !(PAGE_SIZE as u64 - 1);

    let entry = CacheEntry::new(inode, aligned_offset, phys);

    let mut cache = PAGE_CACHE.lock();
    let state = cache.get_or_insert_with(PageCacheState::new);

    // Verifica limite
    if state.max_pages > 0 && state.index.len() >= state.max_pages {
        // Tenta evictar uma página
        if let Some(key) = state.index.evictable_pages(1).first() {
            if let Some(_evicted) = state.index.remove(key.inode, key.offset) {
                PAGES_EVICTED.fetch_add(1, Ordering::Relaxed);
                // TODO: Liberar frame físico via phys::free()
            }
        }
    }

    state.index.insert(entry);
    PAGES_ADDED.fetch_add(1, Ordering::Relaxed);
}

/// Marca página como suja
pub fn mark_dirty(inode: u64, offset: u64) {
    let aligned_offset = offset & !(PAGE_SIZE as u64 - 1);

    let mut cache = PAGE_CACHE.lock();
    if let Some(state) = cache.as_mut() {
        if let Some(entry) = state.index.get_mut(inode, aligned_offset) {
            entry.mark_dirty();
        }
    }
}

/// Invalida todas as páginas de um inode
///
/// Usado após truncate ou delete de arquivo.
pub fn invalidate(inode: u64) {
    let mut cache = PAGE_CACHE.lock();
    if let Some(state) = cache.as_mut() {
        // Remove da fila de writeback
        state.writeback.remove_inode(inode);

        // Remove do índice
        if let Some(pages) = state.index.remove_inode(inode) {
            let count = pages.len();
            PAGES_EVICTED.fetch_add(count as u64, Ordering::Relaxed);

            // TODO: Liberar frames físicos via phys::free()
        }
    }
}

/// Invalida página específica
pub fn invalidate_page(inode: u64, offset: u64) {
    let aligned_offset = offset & !(PAGE_SIZE as u64 - 1);

    let mut cache = PAGE_CACHE.lock();
    if let Some(state) = cache.as_mut() {
        if let Some(_entry) = state.index.remove(inode, aligned_offset) {
            PAGES_EVICTED.fetch_add(1, Ordering::Relaxed);
            // TODO: Liberar frame físico
        }
    }
}

/// Escreve páginas sujas de um inode para disco
///
/// Chamado antes de close ou fsync.
pub fn writeback(inode: u64) {
    let cache = PAGE_CACHE.lock();
    if let Some(state) = cache.as_ref() {
        if let Some(inode_pages) = state.index.get_inode(inode) {
            let dirty = inode_pages.dirty_pages();

            for entry in dirty {
                // Marca início de writeback
                entry.start_writeback();

                // STUB: Escreve para disco
                let _ = writeback::write_page_to_disk(entry.inode(), entry.offset(), entry.phys());

                // Marca fim de writeback
                entry.end_writeback();
            }
        }
    }
}

/// Escreve todas as páginas sujas
pub fn sync_all() {
    let cache = PAGE_CACHE.lock();
    let inodes: alloc::vec::Vec<u64> = if let Some(state) = cache.as_ref() {
        state.index.iter_inodes().map(|(inode, _)| *inode).collect()
    } else {
        alloc::vec::Vec::new()
    };

    drop(cache);

    // Writeback cada inode
    for inode in inodes {
        writeback(inode);
    }
}

/// Retorna número de páginas no cache
pub fn page_count() -> usize {
    PAGE_CACHE
        .lock()
        .as_ref()
        .map(|s| s.index.len())
        .unwrap_or(0)
}

/// Retorna número de inodes no cache
pub fn inode_count() -> usize {
    PAGE_CACHE
        .lock()
        .as_ref()
        .map(|s| s.index.inode_count())
        .unwrap_or(0)
}

// =============================================================================
// Estatísticas
// =============================================================================

/// Estatísticas do cache
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    /// Total de páginas em cache
    pub pages: u64,
    /// Total de inodes
    pub inodes: u64,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Páginas adicionadas
    pub pages_added: u64,
    /// Páginas evictadas
    pub pages_evicted: u64,
    /// Hit ratio (0-100)
    pub hit_ratio: u64,
}

/// Retorna estatísticas do cache
pub fn stats() -> CacheStats {
    let cache = PAGE_CACHE.lock();
    let hits = CACHE_HITS.load(Ordering::Relaxed);
    let misses = CACHE_MISSES.load(Ordering::Relaxed);
    let total = hits + misses;

    let (pages, inodes) = if let Some(state) = cache.as_ref() {
        (state.index.len() as u64, state.index.inode_count() as u64)
    } else {
        (0, 0)
    };

    CacheStats {
        pages,
        inodes,
        hits,
        misses,
        pages_added: PAGES_ADDED.load(Ordering::Relaxed),
        pages_evicted: PAGES_EVICTED.load(Ordering::Relaxed),
        hit_ratio: if total > 0 { (hits * 100) / total } else { 0 },
    }
}

/// Imprime estatísticas
pub fn dump_stats() {
    let s = stats();
    crate::kinfo!("=== Page Cache Statistics ===");
    crate::kinfo!("  Pages: {} ({} inodes)", s.pages, s.inodes);
    crate::kinfo!(
        "  Hits: {}, Misses: {} ({}% hit rate)",
        s.hits,
        s.misses,
        s.hit_ratio
    );
    crate::kinfo!("  Added: {}, Evicted: {}", s.pages_added, s.pages_evicted);
}

// =============================================================================
// Shrinker Integration
// =============================================================================

/// Shrinker para page cache
pub struct PageCacheShrinkerImpl;

impl ShrinkerOps for PageCacheShrinkerImpl {
    fn name(&self) -> &'static str {
        "page_cache"
    }

    fn count(&self) -> usize {
        let cache = PAGE_CACHE.lock();
        cache
            .as_ref()
            .map(|s| s.index.evictable_pages(usize::MAX).len())
            .unwrap_or(0)
    }

    fn shrink(&self, target: usize) -> usize {
        let mut cache = PAGE_CACHE.lock();
        if let Some(state) = cache.as_mut() {
            let evictable = state.index.evictable_pages(target);
            let mut freed = 0;

            for key in evictable {
                if let Some(_entry) = state.index.remove(key.inode, key.offset) {
                    freed += 1;
                    PAGES_EVICTED.fetch_add(1, Ordering::Relaxed);
                    // TODO: Liberar frame físico
                }

                if freed >= target {
                    break;
                }
            }

            freed
        } else {
            0
        }
    }

    fn priority(&self) -> u32 {
        70 // Alta prioridade (fácil de liberar)
    }
}

/// Shrinker estático para registro
pub static PAGE_CACHE_SHRINKER: PageCacheShrinkerImpl = PageCacheShrinkerImpl;

/// Registra shrinker no reclaim
pub fn register_shrinker() {
    crate::rmm::reclaim::register_shrinker(&PAGE_CACHE_SHRINKER);
    crate::kinfo!("(RMM/Cache) Shrinker registrado");
}

// =============================================================================
// Limpar (para testes)
// =============================================================================

/// Limpa todo o cache (CUIDADO: pode causar perda de dados)
#[cfg(debug_assertions)]
pub fn clear_all() {
    let mut cache = PAGE_CACHE.lock();
    if let Some(state) = cache.as_mut() {
        state.index.clear();
        state.writeback.clear();
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_lookup() {
        let inode = 1;
        let offset = 0;
        let phys = PhysAddr::new(0x1000);

        insert(inode, offset, phys);

        let result = lookup(inode, offset);
        assert_eq!(result, Some(phys));
    }

    #[test]
    fn test_cache_invalidate() {
        let inode = 2;

        insert(inode, 0, PhysAddr::new(0x1000));
        insert(inode, 4096, PhysAddr::new(0x2000));

        invalidate(inode);

        assert!(lookup(inode, 0).is_none());
        assert!(lookup(inode, 4096).is_none());
    }

    #[test]
    fn test_cache_mark_dirty() {
        let inode = 3;
        let offset = 0;

        insert(inode, offset, PhysAddr::new(0x3000));
        mark_dirty(inode, offset);

        let cache = PAGE_CACHE.lock();
        let entry = cache.as_ref().unwrap().index.get(inode, offset).unwrap();
        assert!(entry.is_dirty());
    }
}
