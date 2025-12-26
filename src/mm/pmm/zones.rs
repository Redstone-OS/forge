//! # Zonas de Memória Física
//!
//! Divide a memória física em zonas para diferentes usos.
//!
//! ## 🎯 Propósito
//!
//! Diferentes dispositivos e usos requerem memória de regiões específicas:
//! - **DMA Zone** (0-16MB): Dispositivos ISA legados
//! - **DMA32 Zone** (16MB-4GB): Dispositivos com 32-bit addressing
//! - **Normal Zone** (4GB+): Memória geral
//!
//! ## 🏗️ Arquitetura
//!
//! O PMM pode alocar de zonas específicas ou usar fallback:
//! - Pedido de DMA → só DMA zone
//! - Pedido DMA32 → DMA32 ou DMA (fallback)
//! - Pedido Normal → Normal ou DMA32 ou DMA (fallback)
//!
//! ## NUMA
//!
//! Em sistemas NUMA, cada nodo tem suas próprias zonas.

use core::sync::atomic::{AtomicUsize, Ordering};

// =============================================================================
// DEFINIÇÃO DE ZONAS
// =============================================================================

/// Tipo de zona de memória
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ZoneType {
    /// DMA Zone: 0 - 16 MB
    /// Para dispositivos ISA legados que só endereçam 24 bits.
    DMA = 0,

    /// DMA32 Zone: 16 MB - 4 GB
    /// Para dispositivos PCI que só endereçam 32 bits.
    DMA32 = 1,

    /// Normal Zone: 4 GB+
    /// Memória geral para kernel e userspace.
    Normal = 2,

    /// Movable Zone: memória que pode ser migrada/compactada
    /// Usado para huge pages e memory hotplug.
    Movable = 3,
}

impl ZoneType {
    /// Limite superior da zona (exclusivo)
    pub fn upper_limit(&self) -> u64 {
        match self {
            Self::DMA => 16 * 1024 * 1024,         // 16 MB
            Self::DMA32 => 4 * 1024 * 1024 * 1024, // 4 GB
            Self::Normal => u64::MAX,
            Self::Movable => u64::MAX,
        }
    }

    /// Limite inferior da zona (inclusivo)
    pub fn lower_limit(&self) -> u64 {
        match self {
            Self::DMA => 0,
            Self::DMA32 => 16 * 1024 * 1024,
            Self::Normal => 4 * 1024 * 1024 * 1024,
            Self::Movable => 0,
        }
    }

    /// Nome da zona
    pub fn name(&self) -> &'static str {
        match self {
            Self::DMA => "DMA",
            Self::DMA32 => "DMA32",
            Self::Normal => "Normal",
            Self::Movable => "Movable",
        }
    }

    /// Determina a zona para um endereço físico
    pub fn for_address(phys: u64) -> Self {
        if phys < 16 * 1024 * 1024 {
            Self::DMA
        } else if phys < 4 * 1024 * 1024 * 1024 {
            Self::DMA32
        } else {
            Self::Normal
        }
    }

    /// Todas as zonas em ordem
    pub fn all() -> &'static [Self] {
        &[Self::DMA, Self::DMA32, Self::Normal, Self::Movable]
    }
}

// =============================================================================
// ESTATÍSTICAS POR ZONA
// =============================================================================

/// Estatísticas de uma zona de memória
#[repr(C, align(64))]
pub struct ZoneStats {
    /// Frames totais na zona
    pub total_frames: AtomicUsize,
    /// Frames livres
    pub free_frames: AtomicUsize,
    /// Alocações desta zona
    pub alloc_count: AtomicUsize,
    /// Liberações nesta zona
    pub free_count: AtomicUsize,
    /// Alocações que foram fallback para outra zona
    pub fallback_count: AtomicUsize,
}

impl ZoneStats {
    pub const fn new() -> Self {
        Self {
            total_frames: AtomicUsize::new(0),
            free_frames: AtomicUsize::new(0),
            alloc_count: AtomicUsize::new(0),
            free_count: AtomicUsize::new(0),
            fallback_count: AtomicUsize::new(0),
        }
    }
}

// =============================================================================
// ZONA
// =============================================================================

/// Representa uma zona de memória física
pub struct Zone {
    /// Tipo da zona
    zone_type: ZoneType,
    /// Primeiro frame da zona
    start_frame: usize,
    /// Último frame da zona (exclusivo)
    end_frame: usize,
    /// Estatísticas
    stats: ZoneStats,
}

impl Zone {
    /// Cria nova zona
    pub const fn new(zone_type: ZoneType) -> Self {
        Self {
            zone_type,
            start_frame: 0,
            end_frame: 0,
            stats: ZoneStats::new(),
        }
    }

    /// Inicializa zona com range de frames
    pub fn init(&mut self, start: usize, end: usize) {
        self.start_frame = start;
        self.end_frame = end;

        let total = end - start;
        self.stats.total_frames.store(total, Ordering::Relaxed);
        self.stats.free_frames.store(total, Ordering::Relaxed);

        crate::kdebug!(
            "(Zones) {} iniciada: frames {}-{} ({} frames)",
            self.zone_type.name(),
            start,
            end,
            total
        );
    }

    /// Verifica se frame pertence a esta zona
    pub fn contains(&self, frame_idx: usize) -> bool {
        frame_idx >= self.start_frame && frame_idx < self.end_frame
    }

    /// Registra alocação
    pub fn record_alloc(&self) {
        self.stats.free_frames.fetch_sub(1, Ordering::Relaxed);
        self.stats.alloc_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Registra liberação
    pub fn record_free(&self) {
        self.stats.free_frames.fetch_add(1, Ordering::Relaxed);
        self.stats.free_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Frames livres
    pub fn free_frames(&self) -> usize {
        self.stats.free_frames.load(Ordering::Relaxed)
    }

    /// Total de frames
    pub fn total_frames(&self) -> usize {
        self.stats.total_frames.load(Ordering::Relaxed)
    }
}

// =============================================================================
// GERENCIADOR DE ZONAS
// =============================================================================

/// Número de zonas
const NUM_ZONES: usize = 4;

/// Gerenciador global de zonas
pub struct ZoneManager {
    zones: [Zone; NUM_ZONES],
    initialized: bool,
}

impl ZoneManager {
    pub const fn new() -> Self {
        Self {
            zones: [
                Zone::new(ZoneType::DMA),
                Zone::new(ZoneType::DMA32),
                Zone::new(ZoneType::Normal),
                Zone::new(ZoneType::Movable),
            ],
            initialized: false,
        }
    }

    /// Inicializa zonas baseado no mapa de memória
    pub fn init(&mut self, max_phys: u64) {
        use crate::mm::config::PAGE_SIZE;

        let max_frame = (max_phys / PAGE_SIZE as u64) as usize;

        // DMA: 0 - 16MB
        let dma_end = core::cmp::min(16 * 1024 * 1024 / PAGE_SIZE, max_frame);
        self.zones[0].init(0, dma_end);

        // DMA32: 16MB - 4GB
        let dma32_start = dma_end;
        let dma32_end =
            core::cmp::min(4 * 1024 * 1024 * 1024 / PAGE_SIZE as u64, max_frame as u64) as usize;
        if dma32_start < dma32_end {
            self.zones[1].init(dma32_start, dma32_end);
        }

        // Normal: 4GB+
        let normal_start = dma32_end;
        if normal_start < max_frame {
            self.zones[2].init(normal_start, max_frame);
        }

        self.initialized = true;
        crate::kinfo!(
            "(Zones) Inicializado com {} zonas até {}MB",
            NUM_ZONES,
            max_phys / (1024 * 1024)
        );
    }

    /// Obtém zona para um frame
    pub fn zone_for_frame(&self, frame_idx: usize) -> Option<&Zone> {
        for zone in &self.zones {
            if zone.contains(frame_idx) {
                return Some(zone);
            }
        }
        None
    }

    /// Obtém zona por tipo
    pub fn get_zone(&self, zone_type: ZoneType) -> &Zone {
        &self.zones[zone_type as usize]
    }

    /// Imprime estatísticas
    pub fn print_stats(&self) {
        crate::kinfo!("╔═══════════════════════════════════════════╗");
        crate::kinfo!("║          ESTATÍSTICAS DE ZONAS            ║");
        crate::kinfo!("╠════════════╦═══════════╦═══════════╦══════╣");
        crate::kinfo!("║   Zona     ║   Total   ║   Livres  ║  %   ║");
        crate::kinfo!("╠════════════╬═══════════╬═══════════╬══════╣");

        for zone in &self.zones {
            let total = zone.total_frames();
            let free = zone.free_frames();

            if total > 0 {
                let pct = (free * 100) / total;
                crate::kinfo!(
                    "║ {:10} ║ {:>9} ║ {:>9} ║ {:>3}% ║",
                    zone.zone_type.name(),
                    total,
                    free,
                    pct
                );
            }
        }

        crate::kinfo!("╚════════════╩═══════════╩═══════════╩══════╝");
    }
}

/// Gerenciador global
pub static mut ZONE_MANAGER: ZoneManager = ZoneManager::new();

/// Inicializa o sistema de zonas
pub unsafe fn init(max_phys: u64) {
    ZONE_MANAGER.init(max_phys);
}

/// Obtém zona para um frame
pub fn zone_for_frame(frame_idx: usize) -> Option<&'static Zone> {
    unsafe { ZONE_MANAGER.zone_for_frame(frame_idx) }
}
