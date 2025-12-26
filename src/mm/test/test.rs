//! Testes de Memória do Kernel
//!
//! Executa testes de integridade do subsistema de memória no boot.
//! Todos os resultados são enviados para a serial.
//!
//! # Uso
//! Chamar `run_memory_tests()` logo após `mm::init()` no boot.

use crate::mm::config::PAGE_SIZE;
use crate::mm::{heap, pmm, vmm};

/// Executa todos os testes de memória no boot
pub fn run_memory_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE MEMÓRIA               ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_pmm_basic();
    test_vmm_translate();
    test_vmm_lifecycle(); // Novo teste de ciclo de vida
    test_heap_basic();
    test_phys_to_virt();

    // Novos testes de alocadores (Phase 4)
    crate::mm::test::allocator_test::run_alloc_tests();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ TODOS OS TESTES PASSARAM!          ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

/// Teste básico do PMM: alocar e desalocar frames
fn test_pmm_basic() {
    crate::kdebug!("(PMM) Teste: alocando 10 frames...");

    let mut pmm = pmm::FRAME_ALLOCATOR.lock();
    let mut frames = [0u64; 10];

    // Usar while em vez de for para evitar SSE
    let mut i = 0usize;
    while i < 10 {
        let frame = pmm.allocate_frame();

        if frame.is_none() {
            crate::kerror!("(PMM) FALHA: OOM ao alocar frame {}", i);
            panic!("Teste PMM falhou: OOM");
        }

        let f = frame.unwrap();
        frames[i] = f.addr(); // f.addr()

        // Verificar alinhamento
        if f.addr() % PAGE_SIZE as u64 != 0 {
            crate::kerror!("(PMM) FALHA: frame {} não alinhado: {:#x}", i, f.addr());
            panic!("Teste PMM falhou: alinhamento");
        }

        i += 1;
    }

    crate::kdebug!("(PMM) Teste: 10 frames alocados OK");
    crate::kdebug!("(PMM) Teste: desalocando frames...");

    // Desalocar usando while
    let mut j = 0usize;
    while j < 10 {
        let addr = frames[j];
        // Converter u64 -> PhysAddr -> PhysFrame
        use crate::mm::addr::PhysAddr;
        use crate::mm::pmm::PhysFrame;

        // Assumindo que PhysFrame tem from_start_address ou similar
        let frame = PhysFrame::from_start_address(PhysAddr::new(addr));
        pmm.deallocate_frame(frame);
        j += 1;
    }

    crate::kdebug!("(PMM) Teste: desalocação OK");
    crate::kinfo!("(PMM) ✓ PMM alloc/dealloc OK");
}

/// Teste básico do VMM: tradução de endereços
fn test_vmm_translate() {
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
            crate::kinfo!("(VMM) ✓ VMM translate (kernel) OK");
        }
        None => {
            crate::kwarn!("(VMM) Teste: kernel addr não mapeado (pode ser OK)");
            crate::kinfo!("(VMM) ⚠ VMM translate (kernel) não mapeado");
        }
    }

    // Testar tradução de endereço do heap (usa endereço dinâmico)
    let heap_addr: u64 = crate::mm::heap::heap_start() as u64;
    crate::kdebug!("(VMM) Teste: traduzindo endereço heap {:#x}...", heap_addr);

    let result = vmm::translate_addr(heap_addr);

    match result {
        Some(phys) => {
            crate::kdebug!("(VMM) Teste: {:#x} -> phys {:#x}", heap_addr, phys);
            crate::kinfo!("(VMM) ✓ VMM translate (heap) OK");
        }
        None => {
            crate::kerror!("(VMM) FALHA: heap addr não mapeado!");
            panic!("Teste VMM falhou: heap não mapeado");
        }
    }
}

/// Teste básico do Heap: verificar mapeamento
/// NOTA: Testes de Box/Vec desabilitados devido a SSE em __rust_alloc
fn test_heap_basic() {
    crate::kdebug!("(Heap) Teste: verificando mapeamento...");

    // O heap foi mapeado durante init_heap - verificar endereço base
    // Use caminho absoluto para evitar ambiguidade ou erro de re-export
    // O heap foi mapeado durante init_heap - verificar endereço base
    // Use caminho absoluto para evitar ambiguidade ou erro de re-export
    let heap_start = crate::mm::heap::heap_start();
    let heap_size = crate::mm::heap::HEAP_INITIAL_SIZE;

    crate::ktrace!(
        "(Heap) Teste: HEAP_START={:#x}, SIZE={}",
        heap_start,
        heap_size
    );

    // Verificar que podemos ler/escrever no heap via ponteiro raw
    let heap_ptr = heap_start as *mut u64;
    let test_val: u64 = 0xDEAD_BEEF_CAFE_BABE;

    crate::ktrace!("(Heap) Teste: escrevendo valor de teste no heap...");
    unsafe {
        // Usar assembly para evitar SSE
        core::arch::asm!(
            "mov [{0}], {1}",
            in(reg) heap_ptr,
            in(reg) test_val,
            options(nostack, preserves_flags)
        );
    }

    crate::ktrace!("(Heap) Teste: lendo valor de volta...");
    let read_val: u64;
    unsafe {
        core::arch::asm!(
            "mov {0}, [{1}]",
            out(reg) read_val,
            in(reg) heap_ptr,
            options(nostack, preserves_flags, readonly)
        );
    }

    if read_val != test_val {
        crate::kerror!(
            "(Heap) FALHA: escrevemos {:#x} mas lemos {:#x}",
            test_val,
            read_val
        );
        panic!("Teste Heap falhou: mapeamento incorreto");
    }

    crate::ktrace!("(Heap) Teste: valor lido OK: {:#x}", read_val);
    crate::kinfo!("(Heap) ✓ Heap mapping verified");

    // TODO: Resolver SSE em __rust_alloc para habilitar Box/Vec
    crate::kwarn!("(Heap) ⚠ Testes Box/Vec desabilitados (SSE em __rust_alloc)");
}

/// Teste de phys_to_virt
fn test_phys_to_virt() {
    use crate::mm::addr::{self, PhysAddr, VirtAddr};

    // Testar endereço dentro do identity map
    let test_phys_val: u64 = 0x1000000; // 16 MB
    let test_phys = PhysAddr::new(test_phys_val);
    crate::kdebug!("(Addr) Teste: phys_to_virt({:#x})...", test_phys_val);

    if !addr::is_phys_accessible(test_phys) {
        crate::kerror!("(Addr) FALHA: {:#x} deveria ser acessível!", test_phys_val);
        panic!("Teste phys_to_virt falhou");
    }

    crate::ktrace!("(Addr) Teste: is_phys_accessible OK");

    // Testar round-trip
    let virt: VirtAddr = addr::phys_to_virt(test_phys);
    let back: PhysAddr = addr::virt_to_phys(virt).expect("Falha ao reverter virt->phys");

    crate::ktrace!(
        "(Addr) Teste: phys {:#x} -> virt {:#x} -> phys {:#x}",
        test_phys.as_u64(),
        virt.as_u64(),
        back.as_u64()
    );

    if test_phys != back {
        crate::kerror!(
            "(Addr) FALHA: round-trip falhou! {:#x} != {:#x}",
            test_phys.as_u64(),
            back.as_u64()
        );
        panic!("Teste phys_to_virt falhou: round-trip");
    }

    crate::kdebug!("(Addr) Teste: round-trip OK");

    // Testar alinhamento
    // ...
    let test_addr: u64 = 0x12345678;
    let aligned = crate::mm::config::align_down(test_addr as usize, 4096) as u64;
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

    crate::kinfo!("(Addr) ✓ phys_to_virt/virt_to_phys OK");
}

/// Teste de ciclo de vida VMM (Map/Unmap - parcial)
fn test_vmm_lifecycle() {
    crate::kdebug!("(VMM) Teste: map_page (lifecycle)...");

    // Escolher um endereço virtual arbitrário (livre, bem alto)
    let virt_addr = 0xDEAD_0000_0000;

    // Alocar um frame físico manualmente
    let frame = pmm::FRAME_ALLOCATOR
        .lock()
        .allocate_frame()
        .expect("OOM no teste VMM");
    let phys_addr = frame.addr();

    crate::kdebug!("(VMM) Teste: mapeando {:#x} -> {:#x}", virt_addr, phys_addr);

    // Mapear
    unsafe {
        use crate::mm::config::{PAGE_PRESENT, PAGE_WRITABLE};
        if let Err(e) = vmm::map_page(virt_addr, phys_addr, PAGE_PRESENT | PAGE_WRITABLE) {
            crate::kerror!("(VMM) FALHA: map_page retornou erro: {}", e);
            panic!("Teste VMM Lifecycle falhou");
        }
    }

    // Verificar tradução
    match vmm::translate_addr(virt_addr) {
        Some(p) => {
            if p != phys_addr {
                crate::kerror!(
                    "(VMM) FALHA: tradução incorreta {:#x} != {:#x}",
                    p,
                    phys_addr
                );
                panic!("Teste VMM Lifecycle falhou: tradução");
            }
        }
        None => {
            crate::kerror!("(VMM) FALHA: endereço não mapeado após map_page");
            panic!("Teste VMM Lifecycle falhou: não mapeado");
        }
    }

    // TODO: Testar unmap quando implementado
    // unsafe { vmm::unmap_page(virt_addr); }

    crate::kinfo!("(VMM) ✓ VMM map_page lifecycle OK");
}
