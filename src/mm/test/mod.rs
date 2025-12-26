//! # MM Tests
//!
//! Testes do subsistema de memória.

pub mod allocator_test;
pub mod test;

// Testes SMP (TLB shootdown, etc)
#[cfg(feature = "smp")]
pub mod smp;

pub use test::run_memory_tests;
