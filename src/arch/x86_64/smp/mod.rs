//! # SMP Support
//!
//! Módulo principal de SMP.

pub mod tlb;

// Re-exports
pub use tlb::{flush_all, invalidate_page, invalidate_range};
