//! # RMM Statistics
//!
//! Estatísticas expostas do RMM.

/// Estatísticas globais do RMM
#[derive(Debug, Default)]
pub struct RmmStats {
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

impl RmmStats {
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64) / (total as f64) * 100.0
        }
    }
}

/// Coleta estatísticas atuais
pub fn get_stats() -> RmmStats {
    // TODO: Coletar de FrameManager
    RmmStats::default()
}
