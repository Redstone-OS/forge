//! # Zone Manager
//!
//! Gerenciador central de zonas de memória.
//!
//! ## Responsabilidades
//!
//! - Mantém informações sobre cada zona
//! - Rastreia estatísticas (free, used, watermarks)
//! - Provê API para verificação de watermarks
//! - Coordena fallback entre zonas

use super::types::{Zone, ZoneInfo, ZoneStats};
use super::WatermarkLevel;
use crate::rmm::addr::PhysAddr;
use crate::rmm::config::{PAGE_SIZE, ZONE_DMA32_END, ZONE_DMA_END};

// =============================================================================
// ZoneManager
// =============================================================================

/// Gerenciador de zonas de memória
pub struct ZoneManager {
    /// Informações de cada zona
    zones: [ZoneInfo; Zone::COUNT],
    /// Estatísticas de cada zona
    stats: [ZoneStats; Zone::COUNT],
    /// Endereço físico máximo
    max_phys: u64,
}

impl ZoneManager {
    /// Cria novo gerenciador de zonas
    pub fn new(max_phys: u64) -> Self {
        // Calcula limites reais das zonas
        let dma_end = ZONE_DMA_END.min(max_phys);
        let dma32_end = ZONE_DMA32_END.min(max_phys);
        let normal_end = max_phys;

        // Cria ZoneInfo para cada zona
        let zones = [
            ZoneInfo::new(Zone::Dma, PhysAddr::new(0), PhysAddr::new(dma_end)),
            ZoneInfo::new(
                Zone::Dma32,
                PhysAddr::new(dma_end),
                PhysAddr::new(dma32_end),
            ),
            ZoneInfo::new(
                Zone::Normal,
                PhysAddr::new(dma32_end),
                PhysAddr::new(normal_end),
            ),
        ];

        // Inicializa estatísticas
        let stats = [ZoneStats::new(), ZoneStats::new(), ZoneStats::new()];

        for i in 0..Zone::COUNT {
            stats[i].init(zones[i].total_frames);
        }

        Self {
            zones,
            stats,
            max_phys,
        }
    }

    /// Retorna informações de uma zona
    #[inline]
    pub fn zone_info(&self, zone: Zone) -> ZoneInfo {
        self.zones[zone.index()]
    }

    /// Retorna estatísticas de uma zona
    #[inline]
    pub fn stats(&self, zone: Zone) -> &ZoneStats {
        &self.stats[zone.index()]
    }

    /// Atualiza estatísticas de uma zona
    pub fn update_stats(&self, zone: Zone, delta_free: i64, delta_used: i64) {
        self.stats[zone.index()].update(delta_free, delta_used);
    }

    /// Total de frames em todas as zonas
    pub fn total_frames(&self) -> u64 {
        self.zones.iter().map(|z| z.total_frames).sum()
    }

    /// Frames livres em todas as zonas
    pub fn free_frames(&self) -> u64 {
        self.stats
            .iter()
            .map(|s| s.free.load(core::sync::atomic::Ordering::Relaxed))
            .sum()
    }

    /// Frames usados em todas as zonas
    pub fn used_frames(&self) -> u64 {
        self.stats
            .iter()
            .map(|s| s.used.load(core::sync::atomic::Ordering::Relaxed))
            .sum()
    }

    /// Endereço físico máximo
    #[inline]
    pub fn max_phys(&self) -> u64 {
        self.max_phys
    }

    /// Verifica watermark de uma zona
    pub fn check_watermark(&self, zone: Zone) -> WatermarkLevel {
        let stats = &self.stats[zone.index()];

        if stats.below_min() {
            WatermarkLevel::Critical
        } else if stats.below_low() {
            WatermarkLevel::Min
        } else if !stats.above_high() {
            WatermarkLevel::Low
        } else {
            WatermarkLevel::High
        }
    }

    /// Encontra melhor zona para alocação
    ///
    /// Usa lista de fallback se zona preferida está sob pressão.
    pub fn find_zone_for_alloc(&self, preferred: Zone, allow_fallback: bool) -> Option<Zone> {
        // Tenta zona preferida primeiro
        if !self.stats[preferred.index()].below_min() {
            return Some(preferred);
        }

        if !allow_fallback {
            return None;
        }

        // Tenta fallback
        for zone in preferred.fallback_list().iter().skip(1) {
            if !self.stats[zone.index()].below_min() {
                return Some(*zone);
            }
        }

        // Última tentativa: qualquer zona com memória
        for zone in Zone::ALL.iter().rev() {
            if self.stats[zone.index()]
                .free
                .load(core::sync::atomic::Ordering::Relaxed)
                > 0
            {
                return Some(*zone);
            }
        }

        None
    }

    /// Retorna zona com mais memória livre
    pub fn zone_with_most_free(&self) -> Zone {
        let mut best_zone = Zone::Normal;
        let mut best_free = 0u64;

        for zone in Zone::ALL {
            let free = self.stats[zone.index()]
                .free
                .load(core::sync::atomic::Ordering::Relaxed);
            if free > best_free {
                best_free = free;
                best_zone = zone;
            }
        }

        best_zone
    }

    /// Configura watermarks customizados para uma zona
    pub fn set_watermarks(&self, zone: Zone, high: u64, low: u64, min: u64) {
        let stats = &self.stats[zone.index()];
        stats
            .high_watermark
            .store(high, core::sync::atomic::Ordering::Release);
        stats
            .low_watermark
            .store(low, core::sync::atomic::Ordering::Release);
        stats
            .min_watermark
            .store(min, core::sync::atomic::Ordering::Release);
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Calcula número de frames para um tamanho em bytes
#[inline]
pub const fn bytes_to_frames(bytes: u64) -> u64 {
    (bytes + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64
}

/// Calcula bytes a partir de número de frames
#[inline]
pub const fn frames_to_bytes(frames: u64) -> u64 {
    frames * PAGE_SIZE as u64
}

/// Formata tamanho em bytes para string legível
pub fn format_size(bytes: u64) -> (u64, &'static str) {
    if bytes >= 1024 * 1024 * 1024 {
        (bytes / (1024 * 1024 * 1024), "GB")
    } else if bytes >= 1024 * 1024 {
        (bytes / (1024 * 1024), "MB")
    } else if bytes >= 1024 {
        (bytes / 1024, "KB")
    } else {
        (bytes, "B")
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_manager_new() {
        let manager = ZoneManager::new(8 * 1024 * 1024 * 1024); // 8GB

        assert_eq!(manager.zones[0].zone, Zone::Dma);
        assert_eq!(manager.zones[1].zone, Zone::Dma32);
        assert_eq!(manager.zones[2].zone, Zone::Normal);
    }

    #[test]
    fn test_zone_manager_watermarks() {
        let manager = ZoneManager::new(1024 * 1024 * 1024); // 1GB

        // Inicialmente tudo livre, deve estar acima do high
        assert_eq!(manager.check_watermark(Zone::Normal), WatermarkLevel::High);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(1024), (1, "KB"));
        assert_eq!(format_size(1024 * 1024), (1, "MB"));
        assert_eq!(format_size(1024 * 1024 * 1024), (1, "GB"));
    }
}
