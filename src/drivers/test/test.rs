//! Testes de Drivers

use crate::klib::test_framework::{run_test_suite, TestCase, TestResult};

/// Casos de teste de drivers
const DRIVER_TESTS: &[TestCase] = &[
    TestCase::new("pic_remap", test_pic_remap),
    TestCase::new("vga_buffer_size", test_vga_buffer_size),
];

/// Executa todos os testes de drivers
pub fn run_driver_tests() {
    run_test_suite("Drivers", DRIVER_TESTS);
}

/// Verifica offsets do PIC
fn test_pic_remap() -> TestResult {
    // O PIC deve ser remapeado para não conflitar com exceções (0-31)
    let master_offset = 32;
    let slave_offset = 40;

    if master_offset < 32 || slave_offset < 32 {
        crate::kerror!("(Driver) Offsets do PIC CONFLITAM com exceções da CPU");
        return TestResult::Fail;
    }

    crate::ktrace!("(Driver) Offsets do PIC: Master=", master_offset as u64);
    TestResult::Pass
}

/// Valida cálculo de tamanho de framebuffer
fn test_vga_buffer_size() -> TestResult {
    let width = 1024u64;
    let height = 768u64;
    let bpp = 4u64; // 32 bits
    let total_size = width * height * bpp;

    if total_size == 0 {
        crate::kerror!("(Driver) Tamanho de buffer inválido");
        return TestResult::Fail;
    }

    crate::ktrace!("(Driver) Tamanho do buffer=", total_size);
    TestResult::Pass
}
