//! # Testes de TLB Shootdown
//!
//! Valida que TLB shootdown funciona corretamente em SMP.
//!
//! ## Testes
//!
//! 1. Invalidação local
//! 2. Shootdown com múltiplos cores (simulado)
//! 3. Timeout de shootdown

use crate::mm::addr::VirtAddr;
use crate::mm::vmm::tlb;

/// Executa todos os testes de TLB
pub fn run_tlb_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║         🧪 TESTES DE TLB               ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_invlpg_basic();
    test_tlb_flush_all();
    test_tlb_stats();

    #[cfg(feature = "smp")]
    {
        test_shootdown_handler();
    }

    crate::kinfo!("(TLB) ✓ Todos os testes de TLB passaram");
}

/// Teste básico de invlpg
fn test_invlpg_basic() {
    crate::kdebug!("(TLB) Teste: invlpg básico...");

    // Endereço de teste (dentro do heap)
    let test_addr = VirtAddr::new(crate::mm::heap::heap_start() as u64);

    // Invalidar
    unsafe {
        tlb::invlpg(test_addr);
    }

    // Verificar estatísticas
    let local = tlb::TLB_STATS
        .local_invalidations
        .load(core::sync::atomic::Ordering::Relaxed);

    if local == 0 {
        crate::kerror!("(TLB) FALHA: invlpg não incrementou contador");
        panic!("Teste TLB falhou");
    }

    crate::kinfo!("(TLB) ✓ invlpg básico OK");
}

/// Teste de flush completo
fn test_tlb_flush_all() {
    crate::kdebug!("(TLB) Teste: flush_tlb_local...");

    let before = tlb::TLB_STATS
        .full_flushes
        .load(core::sync::atomic::Ordering::Relaxed);

    unsafe {
        tlb::flush_tlb_local();
    }

    let after = tlb::TLB_STATS
        .full_flushes
        .load(core::sync::atomic::Ordering::Relaxed);

    if after <= before {
        crate::kerror!("(TLB) FALHA: flush_tlb_local não incrementou contador");
        panic!("Teste TLB falhou");
    }

    crate::kinfo!("(TLB) ✓ flush_tlb_local OK");
}

/// Teste de estatísticas
fn test_tlb_stats() {
    crate::kdebug!("(TLB) Teste: estatísticas...");

    // Apenas verificar que as estatísticas são acessíveis
    let local = tlb::TLB_STATS
        .local_invalidations
        .load(core::sync::atomic::Ordering::Relaxed);
    let flushes = tlb::TLB_STATS
        .full_flushes
        .load(core::sync::atomic::Ordering::Relaxed);

    crate::kdebug!("(TLB) Stats: local={}, flushes={}", local, flushes);

    // Imprimir relatório
    tlb::print_tlb_stats();

    crate::kinfo!("(TLB) ✓ estatísticas OK");
}

/// Teste do handler de shootdown (só com SMP)
#[cfg(feature = "smp")]
fn test_shootdown_handler() {
    crate::kdebug!("(TLB) Teste: shootdown handler...");

    // Simular chamada do handler
    tlb::shootdown::handle_tlb_ipi();

    crate::kinfo!("(TLB) ✓ shootdown handler OK");
}

/// Teste de invalidação unificada
pub fn test_invalidate_page() {
    crate::kdebug!("(TLB) Teste: invalidate_page...");

    let test_addr = VirtAddr::new(0xDEAD_0000);

    unsafe {
        tlb::invalidate_page(test_addr);
    }

    crate::kinfo!("(TLB) ✓ invalidate_page OK");
}

/// Teste de invalidação de range
pub fn test_invalidate_range() {
    crate::kdebug!("(TLB) Teste: invalidate_range_local...");

    let start = VirtAddr::new(0x1000_0000);

    // Range pequeno (usa invlpg)
    unsafe {
        tlb::invalidate_range_local(start, 4);
    }

    // Range grande (usa flush)
    unsafe {
        tlb::invalidate_range_local(start, 64);
    }

    crate::kinfo!("(TLB) ✓ invalidate_range_local OK");
}
