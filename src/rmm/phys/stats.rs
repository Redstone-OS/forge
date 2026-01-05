//! # Physical Memory Statistics
//!
//! Estatísticas do gerenciador de memória física.
//!
//! ## Métricas Coletadas
//!
//! - Contagem de frames (total, livre, kernel, user, pinned)
//! - Contadores de alocação/liberação
//! - Cache hit/miss rate
//! - Estatísticas por zona
//! - Contention counters (futuro)

use core::sync::atomic::{AtomicU64, Ordering};

/// Estatísticas globais do gerenciador físico
#[repr(C)]
pub struct PhysStats {
    // -------------------------------------------------------------------------
    // Contagem de Frames
    // -------------------------------------------------------------------------
    /// Total de frames gerenciados
    pub total_frames: AtomicU64,
    /// Frames livres atualmente
    pub free_frames: AtomicU64,
    /// Frames alocados para o kernel
    pub kernel_frames: AtomicU64,
    /// Frames alocados para userspace
    pub user_frames: AtomicU64,
    /// Frames pinned (não podem sair da RAM)
    pub pinned_frames: AtomicU64,
    /// Frames em page cache
    pub cache_frames: AtomicU64,
    /// Frames em slab allocator
    pub slab_frames: AtomicU64,

    // -------------------------------------------------------------------------
    // Contadores de Operação
    // -------------------------------------------------------------------------
    /// Total de alocações
    pub alloc_count: AtomicU64,
    /// Total de liberações
    pub free_count: AtomicU64,
    /// Alocações contíguas
    pub contiguous_allocs: AtomicU64,
    /// Falhas de alocação
    pub alloc_failures: AtomicU64,

    // -------------------------------------------------------------------------
    // Cache Per-CPU
    // -------------------------------------------------------------------------
    /// Hits no cache per-CPU
    pub cache_hits: AtomicU64,
    /// Misses no cache per-CPU (slow path)
    pub cache_misses: AtomicU64,
    /// Refills do cache
    pub cache_refills: AtomicU64,
    /// Drains do cache
    pub cache_drains: AtomicU64,

    // -------------------------------------------------------------------------
    // Zonas
    // -------------------------------------------------------------------------
    /// Frames na zona DMA
    pub zone_dma_frames: AtomicU64,
    /// Frames na zona DMA32
    pub zone_dma32_frames: AtomicU64,
    /// Frames na zona Normal
    pub zone_normal_frames: AtomicU64,
    /// Frames livres na zona DMA
    pub zone_dma_free: AtomicU64,
    /// Frames livres na zona DMA32
    pub zone_dma32_free: AtomicU64,
    /// Frames livres na zona Normal
    pub zone_normal_free: AtomicU64,

    // -------------------------------------------------------------------------
    // Compactação e Reclaim
    // -------------------------------------------------------------------------
    /// Páginas movidas por compactação
    pub pages_compacted: AtomicU64,
    /// Páginas reclamadas
    pub pages_reclaimed: AtomicU64,
    /// Páginas swapped out
    pub pages_swapped_out: AtomicU64,
    /// Páginas swapped in
    pub pages_swapped_in: AtomicU64,
}

impl PhysStats {
    /// Cria estatísticas zeradas
    pub const fn new() -> Self {
        Self {
            total_frames: AtomicU64::new(0),
            free_frames: AtomicU64::new(0),
            kernel_frames: AtomicU64::new(0),
            user_frames: AtomicU64::new(0),
            pinned_frames: AtomicU64::new(0),
            cache_frames: AtomicU64::new(0),
            slab_frames: AtomicU64::new(0),

            alloc_count: AtomicU64::new(0),
            free_count: AtomicU64::new(0),
            contiguous_allocs: AtomicU64::new(0),
            alloc_failures: AtomicU64::new(0),

            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_refills: AtomicU64::new(0),
            cache_drains: AtomicU64::new(0),

            zone_dma_frames: AtomicU64::new(0),
            zone_dma32_frames: AtomicU64::new(0),
            zone_normal_frames: AtomicU64::new(0),
            zone_dma_free: AtomicU64::new(0),
            zone_dma32_free: AtomicU64::new(0),
            zone_normal_free: AtomicU64::new(0),

            pages_compacted: AtomicU64::new(0),
            pages_reclaimed: AtomicU64::new(0),
            pages_swapped_out: AtomicU64::new(0),
            pages_swapped_in: AtomicU64::new(0),
        }
    }

    // -------------------------------------------------------------------------
    // Helpers de Registro
    // -------------------------------------------------------------------------

    /// Registra uma alocação
    pub fn record_alloc(&self, from_cache: bool) {
        self.alloc_count.fetch_add(1, Ordering::Relaxed);

        if from_cache {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Registra uma liberação
    pub fn record_free(&self) {
        self.free_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Registra alocação contígua
    pub fn record_contiguous_alloc(&self, count: usize) {
        self.contiguous_allocs.fetch_add(1, Ordering::Relaxed);
        self.alloc_count.fetch_add(count as u64, Ordering::Relaxed);
    }

    /// Registra falha de alocação
    pub fn record_alloc_failure(&self) {
        self.alloc_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Registra refill de cache
    pub fn record_cache_refill(&self, count: usize) {
        self.cache_refills.fetch_add(1, Ordering::Relaxed);
        self.cache_misses.fetch_add(count as u64, Ordering::Relaxed);
    }

    /// Registra drain de cache
    pub fn record_cache_drain(&self, count: usize) {
        self.cache_drains.fetch_add(1, Ordering::Relaxed);
        let _ = count; // Para futuras métricas
    }

    // -------------------------------------------------------------------------
    // Métricas Calculadas
    // -------------------------------------------------------------------------

    /// Taxa de hit do cache (0.0 - 1.0)
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Porcentagem de memória livre
    pub fn free_percentage(&self) -> f64 {
        let total = self.total_frames.load(Ordering::Relaxed);
        let free = self.free_frames.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            (free as f64 / total as f64) * 100.0
        }
    }

    /// Memória usada em bytes
    pub fn used_bytes(&self) -> u64 {
        let total = self.total_frames.load(Ordering::Relaxed);
        let free = self.free_frames.load(Ordering::Relaxed);
        (total.saturating_sub(free)) * crate::rmm::config::PAGE_SIZE as u64
    }

    /// Memória livre em bytes
    pub fn free_bytes(&self) -> u64 {
        self.free_frames.load(Ordering::Relaxed) * crate::rmm::config::PAGE_SIZE as u64
    }

    /// Memória total em bytes
    pub fn total_bytes(&self) -> u64 {
        self.total_frames.load(Ordering::Relaxed) * crate::rmm::config::PAGE_SIZE as u64
    }

    // -------------------------------------------------------------------------
    // Snapshot
    // -------------------------------------------------------------------------

    /// Cria snapshot das estatísticas para exibição
    pub fn snapshot(&self) -> PhysStatsSnapshot {
        PhysStatsSnapshot {
            total_frames: self.total_frames.load(Ordering::Relaxed),
            free_frames: self.free_frames.load(Ordering::Relaxed),
            kernel_frames: self.kernel_frames.load(Ordering::Relaxed),
            user_frames: self.user_frames.load(Ordering::Relaxed),
            pinned_frames: self.pinned_frames.load(Ordering::Relaxed),
            alloc_count: self.alloc_count.load(Ordering::Relaxed),
            free_count: self.free_count.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
        }
    }

    /// Dump para log
    pub fn dump(&self) {
        crate::kinfo!("=== Physical Memory Statistics ===");
        crate::kinfo!(
            "  Frames: {} total, {} free ({:.1}%)",
            self.total_frames.load(Ordering::Relaxed),
            self.free_frames.load(Ordering::Relaxed),
            self.free_percentage()
        );
        crate::kinfo!(
            "  Usage: kernel={}, user={}, pinned={}",
            self.kernel_frames.load(Ordering::Relaxed),
            self.user_frames.load(Ordering::Relaxed),
            self.pinned_frames.load(Ordering::Relaxed)
        );
        crate::kinfo!(
            "  Allocs: {} total, {} failures",
            self.alloc_count.load(Ordering::Relaxed),
            self.alloc_failures.load(Ordering::Relaxed)
        );
        crate::kinfo!(
            "  Cache: {:.1}% hit rate ({} hits, {} misses)",
            self.cache_hit_rate() * 100.0,
            self.cache_hits.load(Ordering::Relaxed),
            self.cache_misses.load(Ordering::Relaxed)
        );
    }
}

impl Default for PhysStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot imutável das estatísticas
#[derive(Debug, Clone, Copy)]
pub struct PhysStatsSnapshot {
    pub total_frames: u64,
    pub free_frames: u64,
    pub kernel_frames: u64,
    pub user_frames: u64,
    pub pinned_frames: u64,
    pub alloc_count: u64,
    pub free_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl PhysStatsSnapshot {
    /// Calcula diferença entre dois snapshots
    pub fn diff(&self, other: &Self) -> PhysStatsDiff {
        PhysStatsDiff {
            alloc_delta: self.alloc_count.saturating_sub(other.alloc_count),
            free_delta: self.free_count.saturating_sub(other.free_count),
            frames_delta: (self.free_frames as i64) - (other.free_frames as i64),
        }
    }
}

/// Diferença entre dois snapshots
#[derive(Debug, Clone, Copy)]
pub struct PhysStatsDiff {
    pub alloc_delta: u64,
    pub free_delta: u64,
    pub frames_delta: i64,
}
