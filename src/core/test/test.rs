//! Testes do Core

use crate::klib::test_framework::{run_test_suite, TestCase, TestResult};

/// Casos de teste do Core
const CORE_TESTS: &[TestCase] = &[
    TestCase::new("boot_magic", test_boot_magic),
    TestCase::new("kernel_address_space", test_kernel_address_space),
];

/// Executa todos os testes do Core
pub fn run_core_tests() {
    run_test_suite("Core", CORE_TESTS);
}

/// Verifica a constante mágica de boot
fn test_boot_magic() -> TestResult {
    use crate::core::handoff::BOOT_MAGIC;

    if BOOT_MAGIC != 0x524544_53544F4E45 {
        crate::kerror!("(Core) Magic INCORRETO: ", BOOT_MAGIC);
        return TestResult::Fail;
    }

    crate::ktrace!("(Core) Magic corresponde a 'REDSTONE'");
    TestResult::Pass
}

/// Valida layout do espaço de endereçamento
fn test_kernel_address_space() -> TestResult {
    let kernel_base = 0xffffffff80000000u64;
    let kernel_top_limit = 0xffffffffffffffffu64;

    if kernel_base >= kernel_top_limit {
        crate::kerror!("(Core) Espaço de endereçamento INVERTIDO");
        return TestResult::Fail;
    }

    crate::ktrace!("(Core) Base do kernel=", kernel_base);
    TestResult::Pass
}
