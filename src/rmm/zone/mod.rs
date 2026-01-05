//! # Zonas de Memória Física
//!
//! Gerenciamento de zonas de memória física do RMM.
//!
//! ## Conceito
//!
//! A memória física é dividida em zonas baseadas no endereço:
//!
//! - **DMA** (0 - 16 MB): Legacy ISA DMA, contiguidade garantida
//! - **DMA32** (16 MB - 4 GB): PCI 32-bit DMA
//! - **Normal** (> 4 GB): Memória de uso geral
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Physical Memory                        │
//! ├─────────────────────────────────────────────────────────────┤
//! │  0x0000_0000 ─────────────────────────────────────────────  │
//! │      │  ZONA DMA (0 - 16 MB)                                │
//! │      │  • Legacy ISA DMA                                    │
//! │      │  • Contiguidade GARANTIDA                            │
//! │  0x0100_0000 ─────────────────────────────────────────────  │
//! │      │  ZONA DMA32 (16 MB - 4 GB)                           │
//! │      │  • PCI 32-bit DMA                                    │
//! │      │  • Contiguidade PREFERIDA                            │
//! │  0x1_0000_0000 ───────────────────────────────────────────  │
//! │      │  ZONA NORMAL (4 GB - MAX)                            │
//! │      │  • Uso geral kernel e userspace                      │
//! │  MAX_PHYS ────────────────────────────────────────────────  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Módulos
//!
//! - `types`: Zone enum, MigrateType, ZoneInfo, ZoneStats
//! - `manager`: ZoneManager para gerenciar zonas

pub mod manager;
pub mod types;

pub use manager::ZoneManager;
pub use types::{MigrateType, Zone, ZoneInfo, ZoneStats};

// Todo: Revisar
#[allow(unused)]
use crate::rmm::addr::PhysAddr;
#[allow(unused)]
use crate::rmm::config::{PAGE_SIZE, ZONE_DMA32_END, ZONE_DMA_END};
use crate::sync::Spinlock;
use core::sync::atomic::{AtomicBool, Ordering};

// =============================================================================
// Estado Global
// =============================================================================

/// Manager global de zonas
static ZONE_MANAGER: Spinlock<Option<ZoneManager>> = Spinlock::new(None);

/// Flag indicando se zonas foram inicializadas
static ZONES_INITIALIZED: AtomicBool = AtomicBool::new(false);

// =============================================================================
// Inicialização
// =============================================================================

/// Inicializa o sistema de zonas
pub fn init(max_phys: u64) {
    crate::kinfo!("(RMM/Zone) Inicializando zonas de memória...");

    let manager = ZoneManager::new(max_phys);

    // Log das zonas
    for zone in [Zone::Dma, Zone::Dma32, Zone::Normal] {
        let info = manager.zone_info(zone);
        let size_mb = (info.size() as usize * PAGE_SIZE) / (1024 * 1024);
        crate::kinfo!(
            "  {}: 0x{:x} - 0x{:x} ({} MB, {} frames)",
            zone.name(),
            info.start.as_u64(),
            info.end.as_u64(),
            size_mb,
            info.total_frames
        );
    }

    *ZONE_MANAGER.lock() = Some(manager);
    ZONES_INITIALIZED.store(true, Ordering::Release);

    crate::kinfo!("(RMM/Zone) Zonas inicializadas");
}

/// Verifica se zonas estão inicializadas
#[inline]
pub fn is_initialized() -> bool {
    ZONES_INITIALIZED.load(Ordering::Acquire)
}

// =============================================================================
// API Pública
// =============================================================================

/// Retorna zona para um endereço físico
#[inline]
pub fn zone_for_address(phys: u64) -> Zone {
    Zone::for_address(phys)
}

/// Retorna informações de uma zona
pub fn get_zone_info(zone: Zone) -> ZoneInfo {
    if let Some(ref manager) = *ZONE_MANAGER.lock() {
        manager.zone_info(zone)
    } else {
        // Fallback se não inicializado
        ZoneInfo::default_for(zone)
    }
}

/// Retorna estatísticas de uma zona
pub fn get_zone_stats(zone: Zone) -> (u64, u64, u64) {
    if let Some(ref manager) = *ZONE_MANAGER.lock() {
        manager.stats(zone).snapshot()
    } else {
        (0, 0, 0)
    }
}

/// Atualiza estatísticas de uma zona (chamado pelo phys allocator)
pub fn update_zone_stats(zone: Zone, delta_free: i64, delta_used: i64) {
    if let Some(ref manager) = *ZONE_MANAGER.lock() {
        manager.update_stats(zone, delta_free, delta_used);
    }
}

/// Retorna número total de frames em todas as zonas
pub fn total_frames() -> u64 {
    if let Some(ref manager) = *ZONE_MANAGER.lock() {
        manager.total_frames()
    } else {
        0
    }
}

/// Retorna número de frames livres em todas as zonas
pub fn free_frames() -> u64 {
    if let Some(ref manager) = *ZONE_MANAGER.lock() {
        manager.free_frames()
    } else {
        0
    }
}

/// Dump de estatísticas de todas as zonas
pub fn dump_stats() {
    crate::kinfo!("=== Zone Statistics ===");

    if let Some(ref manager) = *ZONE_MANAGER.lock() {
        for zone in [Zone::Dma, Zone::Dma32, Zone::Normal] {
            let (total, free, used) = manager.stats(zone).snapshot();
            let usage = if total > 0 { (used * 100) / total } else { 0 };

            crate::kinfo!(
                "  {}: {} free / {} total ({}% used)",
                zone.name(),
                free,
                total,
                usage
            );
        }
    } else {
        crate::kinfo!("  (não inicializado)");
    }
}

// =============================================================================
// Watermarks
// =============================================================================

/// Níveis de pressão de memória
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatermarkLevel {
    /// Memória abundante
    High,
    /// Memória moderada
    Low,
    /// Memória crítica, reclaim urgente
    Min,
    /// Sem memória
    Critical,
}

/// Verifica nível de watermark de uma zona
pub fn check_watermark(zone: Zone) -> WatermarkLevel {
    if let Some(ref manager) = *ZONE_MANAGER.lock() {
        manager.check_watermark(zone)
    } else {
        WatermarkLevel::High
    }
}

/// Verifica se zona está sob pressão
pub fn is_zone_under_pressure(zone: Zone) -> bool {
    matches!(
        check_watermark(zone),
        WatermarkLevel::Low | WatermarkLevel::Min | WatermarkLevel::Critical
    )
}
