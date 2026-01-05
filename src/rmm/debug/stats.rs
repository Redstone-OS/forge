//! # RMM Statistics
//!
//! Coleta e exposição de estatísticas do gerenciador de memória.
//!
//! ## Métricas Coletadas
//!
//! - **Frames**: Total, livre, por tipo de owner
//! - **Zonas**: Estatísticas por zona (DMA, DMA32, Normal)
//! - **Alocações**: Contadores de alloc/free
//! - **Cache**: Hit rate do per-CPU cache
//! - **Reclaim**: Páginas reclamadas/scaneadas

use crate::rmm::config::PAGE_SIZE;
use crate::rmm::zone::Zone;
use core::sync::atomic::{AtomicU64, Ordering};

// =============================================================================
// Contadores Globais
// =============================================================================

/// Contador de alocações
static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
/// Contador de liberações
static FREE_COUNT: AtomicU64 = AtomicU64::new(0);
/// Hits no per-CPU cache
static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
/// Misses no per-CPU cache
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);

// =============================================================================
// ZoneStats
// =============================================================================

/// Estatísticas de uma zona
#[derive(Debug, Clone, Copy, Default)]
pub struct ZoneStats {
    /// Total de frames na zona
    pub total: u64,
    /// Frames livres
    pub free: u64,
    /// Frames movable
    pub movable: u64,
    /// Frames reclaimable
    pub reclaimable: u64,
}

impl ZoneStats {
    /// Memória total em bytes
    #[inline]
    pub fn total_bytes(&self) -> u64 {
        self.total * PAGE_SIZE as u64
    }

    /// Memória livre em bytes
    #[inline]
    pub fn free_bytes(&self) -> u64 {
        self.free * PAGE_SIZE as u64
    }

    /// Percentual usado
    #[inline]
    pub fn used_percent(&self) -> u64 {
        if self.total == 0 {
            0
        } else {
            ((self.total - self.free) * 100) / self.total
        }
    }
}

// =============================================================================
// RmmStats
// =============================================================================

/// Estatísticas globais do RMM
#[derive(Debug, Clone, Default)]
pub struct RmmStats {
    // Frames por categoria
    pub total_frames: u64,
    pub free_frames: u64,
    pub kernel_frames: u64,
    pub user_frames: u64,
    pub cache_frames: u64,
    pub pinned_frames: u64,
    pub shared_frames: u64,
    pub driver_frames: u64,

    // Contadores
    pub alloc_count: u64,
    pub free_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,

    // Por zona
    pub zones: [ZoneStats; 3],

    // Reclaim
    pub pages_reclaimed: u64,
    pub pages_scanned: u64,
}

impl RmmStats {
    /// Nova instância zerada
    pub const fn new() -> Self {
        const ZONE_DEFAULT: ZoneStats = ZoneStats {
            total: 0,
            free: 0,
            movable: 0,
            reclaimable: 0,
        };
        Self {
            total_frames: 0,
            free_frames: 0,
            kernel_frames: 0,
            user_frames: 0,
            cache_frames: 0,
            pinned_frames: 0,
            shared_frames: 0,
            driver_frames: 0,
            alloc_count: 0,
            free_count: 0,
            cache_hits: 0,
            cache_misses: 0,
            zones: [ZONE_DEFAULT; 3],
            pages_reclaimed: 0,
            pages_scanned: 0,
        }
    }

    /// Taxa de hit do cache (0-100)
    pub fn cache_hit_rate(&self) -> u64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0
        } else {
            (self.cache_hits * 100) / total
        }
    }

    /// Memória total em MB
    pub fn total_mb(&self) -> u64 {
        (self.total_frames * PAGE_SIZE as u64) / (1024 * 1024)
    }

    /// Memória livre em MB
    pub fn free_mb(&self) -> u64 {
        (self.free_frames * PAGE_SIZE as u64) / (1024 * 1024)
    }

    /// Memória usada em MB
    pub fn used_mb(&self) -> u64 {
        self.total_mb().saturating_sub(self.free_mb())
    }

    /// Percentual usado
    pub fn used_percent(&self) -> u64 {
        if self.total_frames == 0 {
            0
        } else {
            ((self.total_frames - self.free_frames) * 100) / self.total_frames
        }
    }

    /// Retorna estatísticas de uma zona
    pub fn zone(&self, zone: Zone) -> &ZoneStats {
        match zone {
            Zone::Dma => &self.zones[0],
            Zone::Dma32 => &self.zones[1],
            Zone::Normal => &self.zones[2],
        }
    }
}

// =============================================================================
// API de Coleta
// =============================================================================

/// Coleta estatísticas atuais do RMM
pub fn get_stats() -> RmmStats {
    let mut stats = RmmStats::new();

    // Contadores globais
    stats.alloc_count = ALLOC_COUNT.load(Ordering::Relaxed);
    stats.free_count = FREE_COUNT.load(Ordering::Relaxed);
    stats.cache_hits = CACHE_HITS.load(Ordering::Relaxed);
    stats.cache_misses = CACHE_MISSES.load(Ordering::Relaxed);

    // Coleta do FrameManager
    if let Some(fm_stats) = collect_frame_manager_stats() {
        stats.total_frames = fm_stats.total_frames;
        stats.free_frames = fm_stats.free_frames;
        stats.kernel_frames = fm_stats.kernel_frames;
        stats.user_frames = fm_stats.user_frames;
        stats.cache_frames = fm_stats.cache_frames;
        stats.pinned_frames = fm_stats.pinned_frames;
        stats.shared_frames = fm_stats.shared_frames;
        stats.driver_frames = fm_stats.driver_frames;
        stats.zones = fm_stats.zones;
    } else {
        // Fallback: usa free_count do módulo phys
        stats.free_frames = crate::rmm::phys::free_count() as u64;
    }

    // Coleta do reclaim
    let reclaim_stats = crate::rmm::reclaim::stats();
    stats.pages_reclaimed = reclaim_stats.pages_reclaimed;
    stats.pages_scanned = reclaim_stats.pages_scanned;

    stats
}

/// Coleta estatísticas do FrameManager
fn collect_frame_manager_stats() -> Option<RmmStats> {
    // TODO: Implementar quando FrameManager expuser stats internas
    // Por enquanto retorna None para usar fallback
    None
}

// =============================================================================
// API de Contadores
// =============================================================================

/// Incrementa contador de alocações
#[inline]
pub fn inc_alloc_count() {
    ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Incrementa contador de liberações
#[inline]
pub fn inc_free_count() {
    FREE_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Incrementa contador de cache hits
#[inline]
pub fn inc_cache_hit() {
    CACHE_HITS.fetch_add(1, Ordering::Relaxed);
}

/// Incrementa contador de cache misses
#[inline]
pub fn inc_cache_miss() {
    CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
}

/// Retorna contadores atuais
pub fn counters() -> (u64, u64, u64, u64) {
    (
        ALLOC_COUNT.load(Ordering::Relaxed),
        FREE_COUNT.load(Ordering::Relaxed),
        CACHE_HITS.load(Ordering::Relaxed),
        CACHE_MISSES.load(Ordering::Relaxed),
    )
}

/// Reseta todos os contadores (para testes)
#[cfg(debug_assertions)]
pub fn reset_counters() {
    ALLOC_COUNT.store(0, Ordering::Relaxed);
    FREE_COUNT.store(0, Ordering::Relaxed);
    CACHE_HITS.store(0, Ordering::Relaxed);
    CACHE_MISSES.store(0, Ordering::Relaxed);
}
