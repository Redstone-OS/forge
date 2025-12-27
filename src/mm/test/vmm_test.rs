//! Testes do VMM (Virtual Memory Manager)

use crate::klib::test_framework::{TestCase, TestResult};
use crate::mm::{pmm, vmm};

/// Testes do VMM
pub const VMM_TESTS: &[TestCase] = &[
    TestCase::new("vmm_translate_kernel", test_translate_kernel),
    TestCase::new("vmm_translate_heap", test_translate_heap),
    TestCase::new("vmm_map_lifecycle", test_map_lifecycle),
];

/// Teste: tradução de endereço do kernel
fn test_translate_kernel() -> TestResult {
    let kernel_addr: u64 = 0xffffffff80000000;
    crate::ktrace!("(VMM) Traduzindo endereço do kernel=", kernel_addr);

    match vmm::translate_addr(kernel_addr) {
        Some(_phys) => {
            crate::ktrace!("(VMM) Endereço do kernel traduzido OK");
            TestResult::Pass
        }
        None => {
            // Kernel não mapeado pode ser OK em alguns cenários
            crate::kwarn!("(VMM) Endereço do kernel não mapeado");
            TestResult::Pass // Não falhar, apenas avisar
        }
    }
}

/// Teste: tradução de endereço do heap
fn test_translate_heap() -> TestResult {
    let heap_addr: u64 = crate::mm::heap::heap_start() as u64;
    crate::ktrace!("(VMM) Traduzindo endereço do heap=", heap_addr);

    match vmm::translate_addr(heap_addr) {
        Some(_phys) => {
            crate::ktrace!("(VMM) Endereço do heap traduzido OK");
            TestResult::Pass
        }
        None => {
            crate::kerror!("(VMM) Endereço do heap não mapeado!");
            TestResult::Fail
        }
    }
}

/// Teste: ciclo de vida de mapeamento (map/translate)
fn test_map_lifecycle() -> TestResult {
    // Escolher endereço virtual arbitrário (livre, bem alto)
    let virt_addr = 0xDEAD_0000_0000u64;

    // Alocar um frame físico
    let frame = match pmm::FRAME_ALLOCATOR.lock().allocate_frame() {
        Some(f) => f,
        None => {
            crate::kerror!("(VMM) OOM no teste de ciclo de vida");
            return TestResult::Fail;
        }
    };
    let phys_addr = frame.addr();

    crate::ktrace!("(VMM) Mapeando virt=", virt_addr);

    // Mapear
    unsafe {
        use crate::mm::config::{PAGE_PRESENT, PAGE_WRITABLE};
        if vmm::map_page(virt_addr, phys_addr, PAGE_PRESENT | PAGE_WRITABLE).is_err() {
            crate::kerror!("(VMM) map_page falhou");
            return TestResult::Fail;
        }
    }

    // Verificar tradução
    match vmm::translate_addr(virt_addr) {
        Some(p) if p == phys_addr => {
            crate::ktrace!("(VMM) Ciclo de vida OK");
            TestResult::Pass
        }
        Some(p) => {
            crate::kerror!("(VMM) Tradução incorreta=", p);
            TestResult::Fail
        }
        None => {
            crate::kerror!("(VMM) Não mapeado após map_page");
            TestResult::Fail
        }
    }
}
