//! Testes do Heap

use crate::klib::test_framework::{TestCase, TestResult};

/// Testes do Heap
pub const HEAP_TESTS: &[TestCase] = &[
    TestCase::new("heap_mapping", test_heap_mapping),
    TestCase::new("heap_read_write", test_read_write),
];

/// Teste: verificar que o heap está mapeado
fn test_heap_mapping() -> TestResult {
    let heap_start = crate::mm::heap::heap_start();
    let heap_size = crate::mm::heap::HEAP_INITIAL_SIZE;

    crate::ktrace!("(Heap) HEAP_START=", heap_start);
    crate::ktrace!("(Heap) HEAP_SIZE=", heap_size as u64);

    if heap_start == 0 {
        crate::kerror!("(Heap) Heap não inicializado!");
        return TestResult::Fail;
    }

    TestResult::Pass
}

/// Teste: leitura e escrita no heap
fn test_read_write() -> TestResult {
    let heap_ptr = crate::mm::heap::heap_start() as *mut u64;
    let test_val: u64 = 0xDEAD_BEEF_CAFE_BABE;

    crate::ktrace!("(Heap) Escrevendo valor de teste...");

    unsafe {
        // Escrever valor
        core::ptr::write_volatile(heap_ptr, test_val);

        // Ler de volta
        let read_val = core::ptr::read_volatile(heap_ptr);

        if read_val != test_val {
            crate::kerror!("(Heap) Valor lido incorreto=", read_val);
            return TestResult::Fail;
        }
    }

    crate::ktrace!("(Heap) Leitura/Escrita OK");
    TestResult::Pass
}
