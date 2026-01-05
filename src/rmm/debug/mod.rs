//! # Debug e Observabilidade
//!
//! Utilitários de debug, verificação de integridade e estatísticas.

pub mod dump;
pub mod verify;
pub mod stats;

pub use dump::dump_memory_info;
pub use verify::verify_integrity;
pub use stats::RmmStats;

/// Dump estatísticas do RMM
pub fn dump_stats() {
    crate::kinfo!("=== RMM Statistics ===");
    // TODO: Dump detalhado quando FrameManager estiver implementado
    crate::kinfo!("(RMM/Debug) Stats dumping não implementado ainda");
}
