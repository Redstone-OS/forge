//! Orquestrador de Testes de Memória
//!
//! Executa todos os testes do subsistema de memória usando o framework padronizado.

use crate::klib::test_framework::run_test_suite;

/// Executa todos os testes de memória no boot
pub fn run_memory_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE MEMÓRIA               ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    // PMM Tests
    run_test_suite("PMM", super::pmm_test::PMM_TESTS);

    // VMM Tests
    run_test_suite("VMM", super::vmm_test::VMM_TESTS);

    // Heap Tests
    run_test_suite("Heap", super::heap_test::HEAP_TESTS);

    // Address Tests
    run_test_suite("Addr", super::addr_test::ADDR_TESTS);

    // Allocator Tests (Buddy/Slab)
    super::allocator_test::run_alloc_tests();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ TODOS OS TESTES DE MEMÓRIA OK!     ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}
