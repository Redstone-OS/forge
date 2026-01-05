//! # Page Reclaim
//!
//! Subsistema de reclamação de páginas para liberar memória sob pressão.
//!
//! ## Conceito
//!
//! Quando a memória fica escassa, o kernel precisa liberar páginas.
//! Páginas podem ser:
//!
//! - **Evicted**: Removidas da memória (descartáveis)
//! - **Swapped**: Escritas em disco e removidas
//! - **Migrated**: Movidas para outro nó NUMA
//!
//! ## Algoritmo LRU
//!
//! Usa listas LRU (Least Recently Used) para determinar quais páginas
//! foram acessadas recentemente. Páginas no final da lista são candidatas.
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                      LRU Lists                             │
//! ├────────────────────────────────────────────────────────────┤
//! │                                                            │
//! │  Active List (hot pages)                                   │
//! │  ┌───┬───┬───┬───┬───┬───┐                                 │
//! │  │ A │ B │ C │ D │ E │ F │  ← acessadas recentemente       │
//! │  └───┴───┴───┴───┴───┴───┘                                 │
//! │           ↓ demote                                         │
//! │  Inactive List (cold pages)                                │
//! │  ┌───┬───┬───┬───┬───┬───┐                                 │
//! │  │ G │ H │ I │ J │ K │ L │  ← candidatas para eviction     │
//! │  └───┴───┴───┴───┴───┴───┘                                 │
//! │                       ↓                                    │
//! │                   EVICT/SWAP                               │
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Tipos de Páginas
//!
//! - **Anonymous**: Heap, stack (swap obrigatório)
//! - **File-backed**: mmap, page cache (pode descartar se clean)
//! - **Slab**: Objetos do kernel (shrink caches)

pub mod lru;
pub mod scanner;
pub mod shrinker;

pub use lru::{LruList, LruListType};
pub use scanner::{PageScanner, ScanResult};
pub use shrinker::{Shrinker, ShrinkerOps};

use crate::rmm::addr::PhysAddr;
// Todo: Revisar
#[allow(unused)]
use crate::rmm::config::PAGE_SIZE;
use crate::sync::Spinlock;

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Estado do reclaimer
static RECLAIM_STATE: Spinlock<ReclaimState> = Spinlock::new(ReclaimState::new());

/// Flag indicando se reclaim está ativo
static RECLAIM_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Contador de páginas reclamadas
static PAGES_RECLAIMED: AtomicU64 = AtomicU64::new(0);

/// Contador de páginas scaneadas
static PAGES_SCANNED: AtomicU64 = AtomicU64::new(0);

// =============================================================================
// ReclaimState
// =============================================================================

/// Estado global do reclaimer
struct ReclaimState {
    /// LRU de páginas anônimas
    anon_lru: LruList,
    /// LRU de páginas file-backed
    file_lru: LruList,
    /// Shrinkers registrados
    shrinkers: alloc::vec::Vec<&'static dyn ShrinkerOps>,
    /// Threshold para iniciar reclaim (páginas livres)
    low_watermark: usize,
    /// Threshold para reclaim agressivo
    min_watermark: usize,
    /// Alvo de páginas a reclamar
    reclaim_target: usize,
}

impl ReclaimState {
    const fn new() -> Self {
        Self {
            anon_lru: LruList::new(),
            file_lru: LruList::new(),
            shrinkers: alloc::vec::Vec::new(),
            low_watermark: 1024, // 4MB
            min_watermark: 256,  // 1MB
            reclaim_target: 32,  // 128KB por ciclo
        }
    }
}

// =============================================================================
// API Pública
// =============================================================================

/// Inicializa subsistema de reclaim
pub fn init() {
    crate::kinfo!("(RMM/Reclaim) Inicializando...");

    // Configura watermarks baseado na memória total
    // TODO: Obter memória total do phys manager
    let total_pages = 256 * 1024; // Placeholder: 1GB

    let mut state = RECLAIM_STATE.lock();
    state.low_watermark = total_pages / 16; // 6.25%
    state.min_watermark = total_pages / 64; // 1.5%
    state.reclaim_target = total_pages / 256; // 0.4%

    crate::kinfo!(
        "(RMM/Reclaim) Watermarks: low={}, min={}, target={}",
        state.low_watermark,
        state.min_watermark,
        state.reclaim_target
    );
}

/// Adiciona página à LRU apropriada
pub fn add_to_lru(phys: PhysAddr, is_anon: bool) {
    let mut state = RECLAIM_STATE.lock();
    let lru = if is_anon {
        &mut state.anon_lru
    } else {
        &mut state.file_lru
    };
    lru.add_inactive(phys);
}

/// Remove página das LRUs
pub fn remove_from_lru(phys: PhysAddr) {
    let mut state = RECLAIM_STATE.lock();
    state.anon_lru.remove(phys);
    state.file_lru.remove(phys);
}

/// Marca página como acessada (move para active)
pub fn mark_accessed(phys: PhysAddr, is_anon: bool) {
    let mut state = RECLAIM_STATE.lock();
    let lru = if is_anon {
        &mut state.anon_lru
    } else {
        &mut state.file_lru
    };
    lru.mark_accessed(phys);
}

/// Registra shrinker
pub fn register_shrinker(shrinker: &'static dyn ShrinkerOps) {
    let mut state = RECLAIM_STATE.lock();
    state.shrinkers.push(shrinker);
    crate::kinfo!("(Reclaim) Shrinker registrado: {}", shrinker.name());
}

/// Inicia processo de reclaim
///
/// Retorna número de páginas liberadas.
pub fn reclaim(target: usize) -> usize {
    if RECLAIM_ACTIVE.swap(true, Ordering::AcqRel) {
        // Já está rodando
        return 0;
    }

    let freed = do_reclaim(target);

    RECLAIM_ACTIVE.store(false, Ordering::Release);
    PAGES_RECLAIMED.fetch_add(freed as u64, Ordering::Relaxed);

    freed
}

/// Implementação real do reclaim
fn do_reclaim(target: usize) -> usize {
    let mut freed = 0;
    let mut state = RECLAIM_STATE.lock();

    // 1. Primeiro, tenta shrink de caches (rápido)
    for shrinker in &state.shrinkers {
        let shrunk = shrinker.shrink(target / 4);
        freed += shrunk;
        if freed >= target {
            return freed;
        }
    }

    // 2. Evict páginas file-backed clean (sem I/O)
    let file_target = (target - freed) / 2;
    let file_freed = evict_file_pages(&mut state.file_lru, file_target);
    freed += file_freed;

    if freed >= target {
        return freed;
    }

    // 3. Swap páginas anônimas (com I/O)
    let anon_target = target - freed;
    let anon_freed = swap_anon_pages(&mut state.anon_lru, anon_target);
    freed += anon_freed;

    freed
}

/// Evicta páginas file-backed
fn evict_file_pages(lru: &mut LruList, target: usize) -> usize {
    let mut freed = 0;

    while freed < target {
        if let Some(phys) = lru.pop_inactive() {
            // Verifica se página está limpa
            if can_evict(phys) {
                // Evicta
                if do_evict(phys) {
                    freed += 1;
                } else {
                    // Não conseguiu, devolve para LRU
                    lru.add_inactive(phys);
                }
            } else {
                // Página suja, precisa writeback primeiro
                lru.add_inactive(phys);
            }

            PAGES_SCANNED.fetch_add(1, Ordering::Relaxed);
        } else {
            break; // LRU vazia
        }
    }

    freed
}

/// Swap páginas anônimas
fn swap_anon_pages(lru: &mut LruList, target: usize) -> usize {
    let mut freed = 0;

    while freed < target {
        if let Some(phys) = lru.pop_inactive() {
            // Tenta fazer swap
            if do_swap(phys) {
                freed += 1;
            } else {
                // Não conseguiu, devolve
                lru.add_inactive(phys);
            }

            PAGES_SCANNED.fetch_add(1, Ordering::Relaxed);
        } else {
            break;
        }
    }

    freed
}

/// Verifica se página pode ser evicted (clean file-backed)
// Todo: Revisar
#[allow(unused)]
fn can_evict(phys: PhysAddr) -> bool {
    // TODO: Verificar FrameInfo flags (DIRTY)
    true
}

/// Evicta página (remove mapeamentos e libera)
// Todo: Revisar
#[allow(unused)]
fn do_evict(phys: PhysAddr) -> bool {
    // TODO:
    // 1. Obter rmap da página
    // 2. Remover todos os mapeamentos
    // 3. Liberar página via phys::free()
    false
}

/// Faz swap de página
// Todo: Revisar
#[allow(unused)]
fn do_swap(phys: PhysAddr) -> bool {
    // TODO:
    // 1. Alocar slot no swap
    // 2. Escrever conteúdo da página
    // 3. Atualizar PTEs com entrada de swap
    // 4. Liberar página
    false
}

// =============================================================================
// Watermark Check
// =============================================================================

/// Verifica se memória está sob pressão
pub fn is_under_pressure() -> bool {
    let state = RECLAIM_STATE.lock();
    let free_pages = get_free_pages();
    free_pages < state.low_watermark
}

/// Verifica se reclaim urgente é necessário
pub fn needs_urgent_reclaim() -> bool {
    let state = RECLAIM_STATE.lock();
    let free_pages = get_free_pages();
    free_pages < state.min_watermark
}

/// Retorna número de páginas livres
fn get_free_pages() -> usize {
    // TODO: Obter de phys::stats()
    1024 // Placeholder
}

/// Trigger reclaim se necessário
pub fn check_and_reclaim() -> usize {
    if is_under_pressure() {
        let state = RECLAIM_STATE.lock();
        let target = state.reclaim_target;
        drop(state);
        reclaim(target)
    } else {
        0
    }
}

// =============================================================================
// Estatísticas
// =============================================================================

/// Estatísticas de reclaim
#[derive(Debug, Clone, Copy, Default)]
pub struct ReclaimStats {
    pub pages_reclaimed: u64,
    pub pages_scanned: u64,
    pub anon_active: usize,
    pub anon_inactive: usize,
    pub file_active: usize,
    pub file_inactive: usize,
}

/// Retorna estatísticas
pub fn stats() -> ReclaimStats {
    let state = RECLAIM_STATE.lock();

    ReclaimStats {
        pages_reclaimed: PAGES_RECLAIMED.load(Ordering::Relaxed),
        pages_scanned: PAGES_SCANNED.load(Ordering::Relaxed),
        anon_active: state.anon_lru.active_count(),
        anon_inactive: state.anon_lru.inactive_count(),
        file_active: state.file_lru.active_count(),
        file_inactive: state.file_lru.inactive_count(),
    }
}

/// Dump estatísticas
pub fn dump_stats() {
    let s = stats();
    crate::kinfo!("=== Reclaim Statistics ===");
    crate::kinfo!("  Reclaimed: {} pages", s.pages_reclaimed);
    crate::kinfo!("  Scanned: {} pages", s.pages_scanned);
    crate::kinfo!(
        "  Anon LRU: {} active, {} inactive",
        s.anon_active,
        s.anon_inactive
    );
    crate::kinfo!(
        "  File LRU: {} active, {} inactive",
        s.file_active,
        s.file_inactive
    );
}
