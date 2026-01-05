//! # Debug e Observabilidade
//!
//! Utilitários de debug, verificação de integridade e estatísticas do RMM.
//!
//! ## Componentes
//!
//! - `stats`: Estatísticas globais e por zona
//! - `verify`: Verificação de invariantes
//! - `dump`: Dump de informações de memória
//!
//! ## Uso
//!
//! ```rust,ignore
//! use crate::rmm::debug;
//!
//! // Verificar integridade
//! if debug::verify_integrity() {
//!     kinfo!("RMM integrity OK");
//! }
//!
//! // Dump estatísticas
//! debug::dump_stats();
//!
//! // Dump de um frame específico
//! debug::dump_frame(0x1000);
//! ```
//!
//! ## Visualização
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                    RMM Debug Dashboard                     │
//! ├────────────────────────────────────────────────────────────┤
//! │                      MEMORY OVERVIEW                       │
//! │ ┌────────────────────────────────────────────────────────┐ │
//! │ │ Total: 8192 MB │ Free: 6144 MB │ Used: 2048 MB (25%)   │ │
//! │ └────────────────────────────────────────────────────────┘ │
//! │                                                            │
//! │           ZONES                     ALLOCATION STATS       │
//! │ ┌──────────────────────────┐   ┌─────────────────────────┐ │
//! │ │ DMA:    16 MB (4 free)   │   │ Allocs:  1,234,567      │ │
//! │ │ DMA32:  4 GB  (2 GB free)│   │ Frees:   1,234,000      │ │
//! │ │ Normal: 4 GB  (4 GB free)│   │ Cache Hits: 89%         │ │
//! │ └──────────────────────────┘   └─────────────────────────┘ │
//! │                                                            │
//! │      OWNERSHIP BREAKDOWN                RECLAIM            │
//! │ ┌──────────────────────────┐   ┌─────────────────────────┐ │
//! │ │ Kernel:   512 MB         │   │ Anon Active:  1024      │ │
//! │ │ User:     1024 MB        │   │ Anon Inactive: 512      │ │
//! │ │ Cache:    256 MB         │   │ File Active:   2048     │ │
//! │ │ Pinned:   128 MB         │   │ File Inactive: 1024     │ │
//! │ │ Driver:   64 MB          │   │ Pages Reclaimed: 4096   │ │
//! │ └──────────────────────────┘   └─────────────────────────┘ │
//! └────────────────────────────────────────────────────────────┘
//! ```

pub mod dump;
pub mod stats;
pub mod verify;

pub use dump::{dump_frame, dump_memory_info, dump_zone_info};
pub use stats::{get_stats, RmmStats, ZoneStats};
pub use verify::{verify_frame, verify_integrity, VerifyResult};

use crate::rmm::config::PAGE_SIZE;

// =============================================================================
// API Pública Principal
// =============================================================================

/// Dump completo de estatísticas do RMM
pub fn dump_stats() {
    let stats = get_stats();

    crate::kinfo!("╔═══════════════════════════════════════════════════════════╗");
    crate::kinfo!("║                    RMM Statistics                         ║");
    crate::kinfo!("╠═══════════════════════════════════════════════════════════╣");

    // Memory overview
    let total_mb = (stats.total_frames * PAGE_SIZE as u64) / (1024 * 1024);
    let free_mb = (stats.free_frames * PAGE_SIZE as u64) / (1024 * 1024);
    let used_mb = total_mb.saturating_sub(free_mb);
    let used_pct = if stats.total_frames > 0 {
        ((stats.total_frames - stats.free_frames) * 100) / stats.total_frames
    } else {
        0
    };

    crate::kinfo!(
        "║ Memory: {} MB total, {} MB free, {} MB used ({}%)",
        total_mb,
        free_mb,
        used_mb,
        used_pct
    );
    crate::kinfo!("╠═══════════════════════════════════════════════════════════╣");

    // Zonas
    crate::kinfo!("║ ZONES:");
    for (i, zone) in stats.zones.iter().enumerate() {
        let zone_name = match i {
            0 => "DMA   ",
            1 => "DMA32 ",
            2 => "Normal",
            _ => "???   ",
        };
        let zone_mb = (zone.total * PAGE_SIZE as u64) / (1024 * 1024);
        let zone_free_mb = (zone.free * PAGE_SIZE as u64) / (1024 * 1024);
        crate::kinfo!(
            "║   {}: {} MB total, {} MB free ({} frames)",
            zone_name,
            zone_mb,
            zone_free_mb,
            zone.free
        );
    }
    crate::kinfo!("╠═══════════════════════════════════════════════════════════╣");

    // Ownership
    crate::kinfo!("║ OWNERSHIP:");
    crate::kinfo!(
        "║   Kernel: {} frames ({} MB)",
        stats.kernel_frames,
        (stats.kernel_frames * PAGE_SIZE as u64) / (1024 * 1024)
    );
    crate::kinfo!(
        "║   User:   {} frames ({} MB)",
        stats.user_frames,
        (stats.user_frames * PAGE_SIZE as u64) / (1024 * 1024)
    );
    crate::kinfo!(
        "║   Cache:  {} frames ({} MB)",
        stats.cache_frames,
        (stats.cache_frames * PAGE_SIZE as u64) / (1024 * 1024)
    );
    crate::kinfo!(
        "║   Pinned: {} frames ({} MB)",
        stats.pinned_frames,
        (stats.pinned_frames * PAGE_SIZE as u64) / (1024 * 1024)
    );
    crate::kinfo!("╠═══════════════════════════════════════════════════════════╣");

    // Allocation stats
    crate::kinfo!("║ ALLOCATION:");
    crate::kinfo!("║   Allocs:   {}", stats.alloc_count);
    crate::kinfo!("║   Frees:    {}", stats.free_count);
    crate::kinfo!("║   Cache Hit Rate: {}%", stats.cache_hit_rate());
    crate::kinfo!("╚═══════════════════════════════════════════════════════════╝");
}

/// Verifica integridade e reporta
pub fn check_and_report() -> bool {
    let result = verify_integrity();

    if result.is_ok() {
        crate::kinfo!("(RMM/Debug) Integrity check: PASSED");
        true
    } else {
        crate::kerror!("(RMM/Debug) Integrity check: FAILED");
        crate::kerror!("  Errors: {}", result.error_count);
        for msg in &result.messages {
            crate::kerror!("  - {}", msg);
        }
        false
    }
}

/// Dump rápido de status (para debug interativo)
pub fn quick_status() {
    let stats = get_stats();
    let free_mb = (stats.free_frames * PAGE_SIZE as u64) / (1024 * 1024);
    let total_mb = (stats.total_frames * PAGE_SIZE as u64) / (1024 * 1024);

    crate::kinfo!(
        "(RMM) {} MB free / {} MB total, {} allocs, {} frees",
        free_mb,
        total_mb,
        stats.alloc_count,
        stats.free_count
    );
}
