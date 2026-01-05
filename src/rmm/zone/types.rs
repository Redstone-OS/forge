//! # Tipos de Zona e MigrateType
//!
//! Define Zone (DMA, DMA32, Normal) e MigrateType (Unmovable, Movable, Reclaimable).

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::{ZONE_DMA32_END, ZONE_DMA_END};
use core::sync::atomic::{AtomicU64, Ordering};

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

    /// Determina a zona de um endereço físico
    #[inline]
    pub const fn from_phys(phys: PhysAddr) -> Self {
        let addr = phys.as_u64();
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
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Dma => "DMA",
            Self::Dma32 => "DMA32",
            Self::Normal => "Normal",
        }
    }
}

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
    pub const COUNT: usize = 4;

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Unmovable => "Unmovable",
            Self::Movable => "Movable",
            Self::Reclaimable => "Reclaimable",
            Self::Cma => "CMA",
        }
    }
}

/// Informações de uma zona
pub struct ZoneInfo {
    pub zone: Zone,
    pub start: PhysAddr,
    pub end: PhysAddr,
    pub total_frames: usize,
}

/// Estatísticas de uma zona
pub struct ZoneStats {
    pub total: AtomicU64,
    pub free: AtomicU64,
    pub used: AtomicU64,
}

impl ZoneStats {
    pub const fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            free: AtomicU64::new(0),
            used: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> (u64, u64, u64) {
        (
            self.total.load(Ordering::Relaxed),
            self.free.load(Ordering::Relaxed),
            self.used.load(Ordering::Relaxed),
        )
    }
}
