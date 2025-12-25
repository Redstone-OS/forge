//! Testes de Memória do Kernel
//!
//! Executa testes de integridade do subsistema de memória no boot.
//! Todos os resultados são enviados para a serial.
//!
//! # Uso
//! Chamar `run_memory_tests()` logo após `mm::init()` no boot.

use crate::mm::pmm::FRAME_SIZE;
use crate::mm::{heap, pmm, vmm};

/// Executa todos os testes de memória no boot
pub fn run_memory_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE MEMÓRIA               ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_pmm_basic();
    test_vmm_translate();
    test_heap_basic();
    test_phys_to_virt();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ TODOS OS TESTES PASSARAM!          ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

/// Teste básico do PMM: alocar e desalocar frames
fn test_pmm_basic() {
    crate::kinfo!("┌─ Teste PMM ─────────────────────────────┐");
    crate::kdebug!("(PMM) Teste: alocando 10 frames...");

    let mut pmm = pmm::FRAME_ALLOCATOR.lock();
    let mut frames = [0u64; 10];

    for i in 0..10 {
        crate::ktrace!("(PMM) Teste: alocando frame {}...", i);
        let frame = pmm.allocate_frame();

        if frame.is_none() {
            crate::kerror!("(PMM) FALHA: OOM ao alocar frame {}", i);
            panic!("Teste PMM falhou: OOM");
        }

        let f = frame.unwrap();
        frames[i] = f.addr;

        // Verificar alinhamento
        if f.addr % FRAME_SIZE as u64 != 0 {
            crate::kerror!("(PMM) FALHA: frame {} não alinhado: {:#x}", i, f.addr);
            panic!("Teste PMM falhou: alinhamento");
        }

        crate::ktrace!("(PMM) Teste: frame {} = {:#x}", i, f.addr);
    }

    crate::kdebug!("(PMM) Teste: 10 frames alocados OK");
    crate::kdebug!("(PMM) Teste: desalocando frames...");

    // Desalocar
    for (i, &addr) in frames.iter().enumerate() {
        crate::ktrace!("(PMM) Teste: desalocando frame {} ({:#x})...", i, addr);
        pmm.deallocate_frame((addr / FRAME_SIZE as u64) as usize);
    }

    crate::kdebug!("(PMM) Teste: desalocação OK");
    crate::kinfo!("│  ✓ PMM alloc/dealloc OK                 │");
    crate::kinfo!("└──────────────────────────────────────────┘");
}

/// Teste básico do VMM: tradução de endereços
fn test_vmm_translate() {
    crate::kinfo!("┌─ Teste VMM ─────────────────────────────┐");

    // Testar tradução de endereço do kernel (deve funcionar)
    let kernel_addr: u64 = 0xffffffff80000000;
    crate::kdebug!(
        "(VMM) Teste: traduzindo endereço kernel {:#x}...",
        kernel_addr
    );

    let result = vmm::translate_addr(kernel_addr);

    match result {
        Some(phys) => {
            crate::kdebug!("(VMM) Teste: {:#x} -> phys {:#x}", kernel_addr, phys);
            crate::kinfo!("│  ✓ VMM translate (kernel) OK           │");
        }
        None => {
            crate::kwarn!("(VMM) Teste: kernel addr não mapeado (pode ser OK)");
            crate::kinfo!("│  ⚠ VMM translate (kernel) não mapeado  │");
        }
    }

    // Testar tradução de endereço do heap
    let heap_addr: u64 = heap::HEAP_START as u64;
    crate::kdebug!("(VMM) Teste: traduzindo endereço heap {:#x}...", heap_addr);

    let result = vmm::translate_addr(heap_addr);

    match result {
        Some(phys) => {
            crate::kdebug!("(VMM) Teste: {:#x} -> phys {:#x}", heap_addr, phys);
            crate::kinfo!("│  ✓ VMM translate (heap) OK             │");
        }
        None => {
            crate::kerror!("(VMM) FALHA: heap addr não mapeado!");
            panic!("Teste VMM falhou: heap não mapeado");
        }
    }

    crate::kinfo!("└──────────────────────────────────────────┘");
}

/// Teste básico do Heap: alocar e verificar integridade
fn test_heap_basic() {
    crate::kinfo!("┌─ Teste Heap ────────────────────────────┐");
    crate::kdebug!("(Heap) Teste: alocando Vec<u64> com 1024 elementos...");

    use alloc::vec::Vec;

    // Alocar vetor
    let mut v: Vec<u64> = Vec::with_capacity(1024);
    crate::ktrace!("(Heap) Teste: Vec::with_capacity OK, ptr={:p}", v.as_ptr());

    // Preencher
    for i in 0..1024 {
        v.push(i as u64);
        if i % 256 == 0 {
            crate::ktrace!("(Heap) Teste: preenchido até índice {}", i);
        }
    }

    crate::kdebug!("(Heap) Teste: preenchimento OK, verificando integridade...");

    // Verificar
    for (i, &val) in v.iter().enumerate() {
        if val != i as u64 {
            crate::kerror!("(Heap) FALHA: v[{}] = {} (esperado {})", i, val, i);
            panic!("Teste Heap falhou: corrupção");
        }
    }

    crate::kdebug!("(Heap) Teste: integridade OK");

    // Testar String
    crate::ktrace!("(Heap) Teste: alocando String...");
    use alloc::string::String;
    let s = String::from("Redstone OS - Teste de Memória OK!");
    crate::ktrace!("(Heap) Teste: String OK, len={}", s.len());

    crate::kinfo!("│  ✓ Heap alloc/integrity OK              │");
    crate::kinfo!("└──────────────────────────────────────────┘");
}

/// Teste de phys_to_virt
fn test_phys_to_virt() {
    crate::kinfo!("┌─ Teste phys_to_virt ────────────────────┐");

    use crate::mm::addr;

    // Testar endereço dentro do identity map
    let test_phys: u64 = 0x1000000; // 16 MB
    crate::kdebug!("(Addr) Teste: phys_to_virt({:#x})...", test_phys);

    if !addr::is_phys_accessible(test_phys) {
        crate::kerror!("(Addr) FALHA: {:#x} deveria ser acessível!", test_phys);
        panic!("Teste phys_to_virt falhou");
    }

    crate::ktrace!("(Addr) Teste: is_phys_accessible OK");

    // Testar round-trip
    let virt = unsafe { addr::phys_to_virt::<u8>(test_phys) };
    let back = addr::virt_to_phys(virt);

    crate::ktrace!(
        "(Addr) Teste: phys {:#x} -> virt {:p} -> phys {:#x}",
        test_phys,
        virt,
        back
    );

    if test_phys != back {
        crate::kerror!(
            "(Addr) FALHA: round-trip falhou! {:#x} != {:#x}",
            test_phys,
            back
        );
        panic!("Teste phys_to_virt falhou: round-trip");
    }

    crate::kdebug!("(Addr) Teste: round-trip OK");

    // Testar alinhamento
    let test_addr: u64 = 0x12345678;
    let aligned = addr::frame_align_down(test_addr);
    let expected: u64 = 0x12345000;

    crate::ktrace!(
        "(Addr) Teste: frame_align_down({:#x}) = {:#x}",
        test_addr,
        aligned
    );

    if aligned != expected {
        crate::kerror!(
            "(Addr) FALHA: alinhamento errado! {:#x} != {:#x}",
            aligned,
            expected
        );
        panic!("Teste frame_align falhou");
    }

    crate::kinfo!("│  ✓ phys_to_virt/virt_to_phys OK         │");
    crate::kinfo!("└──────────────────────────────────────────┘");
}
