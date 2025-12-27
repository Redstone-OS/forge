//! # Testes do Subsistema de Memória
//!
//! Organização modular dos testes:
//! - `pmm_test.rs` - Testes do Physical Memory Manager
//! - `vmm_test.rs` - Testes do Virtual Memory Manager
//! - `heap_test.rs` - Testes do Heap
//! - `addr_test.rs` - Testes de conversão de endereços
//! - `allocator_test.rs` - Testes dos alocadores (Buddy/Slab)

pub mod addr_test;
pub mod allocator_test;
pub mod heap_test;
pub mod pmm_test;
pub mod vmm_test;

// Manter compatibilidade com código antigo
pub mod test;

// Testes SMP (TLB shootdown, etc)
#[cfg(feature = "smp")]
pub mod smp;

// Re-export da função principal
pub use test::run_memory_tests;
