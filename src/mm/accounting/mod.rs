//! # Memory Accounting - Módulo Principal
//!
//! Rastreamento de uso de memória por subsistema para:
//! - Diagnóstico de vazamentos
//! - Aplicação de quotas
//! - Isolamento de falhas
//!
//! ## 🎯 Propósito
//!
//! Quando um driver ou subsistema vaza memória, é difícil identificar
//! a origem sem tracking. Este módulo permite:
//!
//! 1. Associar cada alocação a um subsistema
//! 2. Definir quotas (soft/hard limits)
//! 3. Gerar relatórios de uso
//!
//! ## 🏗️ Arquitetura
//!
//! - Cada task/thread tem um subsistema "atual"
//! - Alocações são contabilizadas no subsistema atual
//! - Quotas podem bloquear alocações excessivas
//!
//! ## 🔧 Uso
//!
//! ```rust
//! // Definir contexto de subsistema
//! accounting::set_current_subsystem(Subsystem::Network);
//!
//! // Alocações são contabilizadas em Network
//! let buffer = vec![0u8; 4096];
//!
//! // Ver relatório
//! accounting::print_memory_report();
//! ```

pub mod stats;
pub mod subsystem;

pub use stats::{get_stats, print_memory_report, SubsystemStats};
pub use subsystem::{get_current_subsystem, set_current_subsystem, Subsystem};

// =============================================================================
// RE-EXPORTS
// =============================================================================

/// Registra alocação no subsistema atual
pub fn record_alloc(bytes: usize) -> bool {
    let subsys = get_current_subsystem();
    get_stats(subsys).record_alloc(bytes)
}

/// Registra liberação no subsistema atual
pub fn record_free(bytes: usize) {
    let subsys = get_current_subsystem();
    get_stats(subsys).record_free(bytes);
}

/// Define quota para um subsistema
pub fn set_quota(subsys: Subsystem, bytes: usize) {
    get_stats(subsys).set_quota(bytes);
}

/// Obtém uso atual de um subsistema
pub fn get_usage(subsys: Subsystem) -> usize {
    get_stats(subsys).allocated_bytes()
}

// =============================================================================
// INTEGRAÇÃO COM ALLOCATOR
// =============================================================================

/// Helper para integrar com o allocator
///
/// Chame isso em wrapper de alloc quando memory_accounting está habilitado.
#[cfg(feature = "memory_accounting")]
pub fn on_alloc(size: usize) -> bool {
    record_alloc(size)
}

#[cfg(not(feature = "memory_accounting"))]
pub fn on_alloc(_size: usize) -> bool {
    true
}

#[cfg(feature = "memory_accounting")]
pub fn on_free(size: usize) {
    record_free(size);
}

#[cfg(not(feature = "memory_accounting"))]
pub fn on_free(_size: usize) {}
