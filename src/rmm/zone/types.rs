//! # Tipos de Zona e MigrateType
//!
//! Define Zone (DMA, DMA32, Normal) e MigrateType (Unmovable, Movable, Reclaimable).
//!
//! ## Zone
//!
//! Zonas são regiões de memória física com características específicas:
//!
//! | Zona | Range | Uso |
//! |------|-------|-----|
//! | DMA | 0 - 16MB | Legacy ISA DMA |
//! | DMA32 | 16MB - 4GB | PCI 32-bit DMA |
//! | Normal | > 4GB | Uso geral |
//!
//! ## MigrateType
//!
//! Categoriza frames por mobilidade para compactação:
//!
//! | Tipo | Descrição |
//! |------|-----------|
//! | Unmovable | Page tables, kernel stacks |
//! | Movable | Userspace, pode compactar |
//! | Reclaimable | Page cache, pode descartar |
//! | CMA | Contiguous Memory Allocator |

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::{PAGE_SIZE, ZONE_DMA32_END, ZONE_DMA_END};
use core::sync::atomic::{AtomicU64, Ordering};

// =============================================================================
// Zone
// =============================================================================

/// Zona de memória física
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Zone {
    /// 0 - 16 MB: Legacy ISA DMA
    Dma = 0,
    /// 16 MB - 4 GB: PCI 32-bit DMA
    Dma32 = 1,
    /// > 4 GB: Memória normal
    Normal = 2,
}

impl Zone {
    /// Número total de zonas
    pub const COUNT: usize = 3;

    /// Todas as zonas em ordem
    pub const ALL: [Zone; 3] = [Zone::Dma, Zone::Dma32, Zone::Normal];

    /// Determina a zona de um endereço físico
    #[inline]
    pub const fn from_phys(phys: PhysAddr) -> Self {
        Self::for_address(phys.as_u64())
    }

    /// Determina a zona de um endereço físico (u64)
    #[inline]
    pub const fn for_address(addr: u64) -> Self {
        if addr < ZONE_DMA_END {
            Self::Dma
        } else if addr < ZONE_DMA32_END {
            Self::Dma32
        } else {
            Self::Normal
        }
    }

    /// Converte de índice
    #[inline]
    pub const fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Dma,
            1 => Self::Dma32,
            _ => Self::Normal,
        }
    }

    /// Retorna índice da zona
    #[inline]
    pub const fn index(&self) -> usize {
        *self as usize
    }

    /// Nome da zona
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Dma => "DMA",
            Self::Dma32 => "DMA32",
            Self::Normal => "Normal",
        }
    }

    /// Retorna range de endereços da zona
    #[inline]
    pub const fn address_range(&self) -> (u64, u64) {
        match self {
            Self::Dma => (0, ZONE_DMA_END),
            Self::Dma32 => (ZONE_DMA_END, ZONE_DMA32_END),
            Self::Normal => (ZONE_DMA32_END, u64::MAX),
        }
    }

    /// Verifica se endereço pertence a esta zona
    #[inline]
    pub const fn contains(&self, addr: u64) -> bool {
        let (start, end) = self.address_range();
        addr >= start && addr < end
    }

    /// Retorna zona de fallback (para quando esta está cheia)
    pub const fn fallback(&self) -> Option<Self> {
        match self {
            Self::Dma => Some(Self::Dma32),
            Self::Dma32 => Some(Self::Normal),
            Self::Normal => None,
        }
    }

    /// Lista de zonas em ordem de preferência para fallback
    pub const fn fallback_list(&self) -> &'static [Zone] {
        match self {
            Self::Dma => &[Zone::Dma, Zone::Dma32, Zone::Normal],
            Self::Dma32 => &[Zone::Dma32, Zone::Normal],
            Self::Normal => &[Zone::Normal],
        }
    }
}

impl core::fmt::Display for Zone {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// MigrateType
// =============================================================================

/// Tipo de migração do frame
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MigrateType {
    /// Não pode ser movido (page tables, kernel stacks)
    Unmovable = 0,
    /// Pode ser movido (userspace)
    Movable = 1,
    /// Pode ser descartado (page cache)
    Reclaimable = 2,
    /// Reservado para CMA
    Cma = 3,
}

impl MigrateType {
    /// Número de tipos
    pub const COUNT: usize = 4;

    /// Nome do tipo
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Unmovable => "Unmovable",
            Self::Movable => "Movable",
            Self::Reclaimable => "Reclaimable",
            Self::Cma => "CMA",
        }
    }

    /// Pode ser compactado?
    #[inline]
    pub const fn is_movable(&self) -> bool {
        matches!(self, Self::Movable)
    }

    /// Pode ser reclamado sem swap?
    #[inline]
    pub const fn is_reclaimable(&self) -> bool {
        matches!(self, Self::Reclaimable)
    }

    /// Converte de índice
    #[inline]
    pub const fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Unmovable,
            1 => Self::Movable,
            2 => Self::Reclaimable,
            3 => Self::Cma,
            _ => Self::Unmovable,
        }
    }

    /// Retorna índice
    #[inline]
    pub const fn index(&self) -> usize {
        *self as usize
    }
}

impl core::fmt::Display for MigrateType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// ZoneInfo
// =============================================================================

/// Informações de uma zona
#[derive(Debug, Clone, Copy)]
pub struct ZoneInfo {
    /// Tipo da zona
    pub zone: Zone,
    /// Endereço inicial
    pub start: PhysAddr,
    /// Endereço final (exclusivo)
    pub end: PhysAddr,
    /// Total de frames na zona
    pub total_frames: u64,
}

impl ZoneInfo {
    /// Cria ZoneInfo
    pub const fn new(zone: Zone, start: PhysAddr, end: PhysAddr) -> Self {
        let size = end.as_u64().saturating_sub(start.as_u64());
        let total_frames = size / PAGE_SIZE as u64;

        Self {
            zone,
            start,
            end,
            total_frames,
        }
    }

    /// Cria ZoneInfo padrão para uma zona
    pub fn default_for(zone: Zone) -> Self {
        let (start, end) = zone.address_range();
        Self::new(zone, PhysAddr::new(start), PhysAddr::new(end))
    }

    /// Tamanho em frames
    #[inline]
    pub const fn size(&self) -> u64 {
        self.total_frames
    }

    /// Tamanho em bytes
    #[inline]
    pub const fn size_bytes(&self) -> u64 {
        self.total_frames * PAGE_SIZE as u64
    }

    /// Verifica se endereço está nesta zona
    #[inline]
    pub fn contains(&self, addr: PhysAddr) -> bool {
        addr.as_u64() >= self.start.as_u64() && addr.as_u64() < self.end.as_u64()
    }
}

// =============================================================================
// ZoneStats
// =============================================================================

/// Estatísticas de uma zona (atômicas para concorrência)
pub struct ZoneStats {
    /// Total de frames
    pub total: AtomicU64,
    /// Frames livres
    pub free: AtomicU64,
    /// Frames em uso
    pub used: AtomicU64,
    /// Frames reservados
    pub reserved: AtomicU64,
    /// High watermark (threshold para reclaim)
    pub high_watermark: AtomicU64,
    /// Low watermark
    pub low_watermark: AtomicU64,
    /// Min watermark (crítico)
    pub min_watermark: AtomicU64,
}

impl ZoneStats {
    /// Cria stats zeradas
    pub const fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            free: AtomicU64::new(0),
            used: AtomicU64::new(0),
            reserved: AtomicU64::new(0),
            high_watermark: AtomicU64::new(0),
            low_watermark: AtomicU64::new(0),
            min_watermark: AtomicU64::new(0),
        }
    }

    /// Inicializa com valores
    pub fn init(&self, total: u64) {
        self.total.store(total, Ordering::Release);
        self.free.store(total, Ordering::Release);
        self.used.store(0, Ordering::Release);

        // Watermarks padrão
        self.high_watermark.store(total / 4, Ordering::Release); // 25%
        self.low_watermark.store(total / 8, Ordering::Release); // 12.5%
        self.min_watermark.store(total / 16, Ordering::Release); // 6.25%
    }

    /// Snapshot dos valores atuais
    pub fn snapshot(&self) -> (u64, u64, u64) {
        (
            self.total.load(Ordering::Relaxed),
            self.free.load(Ordering::Relaxed),
            self.used.load(Ordering::Relaxed),
        )
    }

    /// Atualiza contadores
    pub fn update(&self, delta_free: i64, delta_used: i64) {
        if delta_free > 0 {
            self.free.fetch_add(delta_free as u64, Ordering::Relaxed);
        } else if delta_free < 0 {
            self.free.fetch_sub((-delta_free) as u64, Ordering::Relaxed);
        }

        if delta_used > 0 {
            self.used.fetch_add(delta_used as u64, Ordering::Relaxed);
        } else if delta_used < 0 {
            self.used.fetch_sub((-delta_used) as u64, Ordering::Relaxed);
        }
    }

    /// Incrementa free, decrementa used
    pub fn mark_freed(&self) {
        self.free.fetch_add(1, Ordering::Relaxed);
        self.used.fetch_sub(1, Ordering::Relaxed);
    }

    /// Decrementa free, incrementa used
    pub fn mark_allocated(&self) {
        self.free.fetch_sub(1, Ordering::Relaxed);
        self.used.fetch_add(1, Ordering::Relaxed);
    }

    /// Verifica se está acima do high watermark
    pub fn above_high(&self) -> bool {
        self.free.load(Ordering::Relaxed) > self.high_watermark.load(Ordering::Relaxed)
    }

    /// Verifica se está abaixo do low watermark
    pub fn below_low(&self) -> bool {
        self.free.load(Ordering::Relaxed) < self.low_watermark.load(Ordering::Relaxed)
    }

    /// Verifica se está abaixo do min watermark
    pub fn below_min(&self) -> bool {
        self.free.load(Ordering::Relaxed) < self.min_watermark.load(Ordering::Relaxed)
    }

    /// Percentual de uso
    pub fn usage_percent(&self) -> u64 {
        let total = self.total.load(Ordering::Relaxed);
        if total == 0 {
            0
        } else {
            (self.used.load(Ordering::Relaxed) * 100) / total
        }
    }
}

impl Default for ZoneStats {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_for_address() {
        assert_eq!(Zone::for_address(0), Zone::Dma);
        assert_eq!(Zone::for_address(1024 * 1024), Zone::Dma); // 1MB
        assert_eq!(Zone::for_address(ZONE_DMA_END - 1), Zone::Dma);
        assert_eq!(Zone::for_address(ZONE_DMA_END), Zone::Dma32);
        assert_eq!(Zone::for_address(ZONE_DMA32_END - 1), Zone::Dma32);
        assert_eq!(Zone::for_address(ZONE_DMA32_END), Zone::Normal);
    }

    #[test]
    fn test_zone_contains() {
        assert!(Zone::Dma.contains(0));
        assert!(Zone::Dma.contains(1000));
        assert!(!Zone::Dma.contains(ZONE_DMA_END));
    }

    #[test]
    fn test_zone_stats() {
        let stats = ZoneStats::new();
        stats.init(1000);

        assert_eq!(stats.snapshot(), (1000, 1000, 0));

        stats.mark_allocated();
        assert_eq!(stats.snapshot(), (1000, 999, 1));

        stats.mark_freed();
        assert_eq!(stats.snapshot(), (1000, 1000, 0));
    }
}
