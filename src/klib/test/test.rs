//! Testes da Biblioteca klib

use crate::klib::test_framework::{run_test_suite, TestCase, TestResult};

/// Testes de alinhamento
const ALIGN_TESTS: &[TestCase] = &[
    TestCase::new("align_up", test_align_up),
    TestCase::new("align_down", test_align_down),
    TestCase::new("is_aligned", test_is_aligned),
];

/// Testes de manipulação de bits
const BITS_TESTS: &[TestCase] = &[TestCase::new("bit_set_clear", test_bit_set_clear)];

/// Executa todos os testes de klib
pub fn run_klib_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE KLIB                  ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    run_test_suite("Align", ALIGN_TESTS);
    run_test_suite("Bits", BITS_TESTS);

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ KLIB VALIDADO!                     ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

// =============================================================================
// TESTES DE ALINHAMENTO
// =============================================================================

fn test_align_up() -> TestResult {
    use crate::klib::align_up;

    // Casos de teste
    if align_up(0, 4) != 0 {
        return TestResult::Fail;
    }
    if align_up(1, 4) != 4 {
        return TestResult::Fail;
    }
    if align_up(4, 4) != 4 {
        return TestResult::Fail;
    }
    if align_up(5, 4) != 8 {
        return TestResult::Fail;
    }
    if align_up(4097, 4096) != 8192 {
        return TestResult::Fail;
    }

    TestResult::Pass
}

fn test_align_down() -> TestResult {
    use crate::klib::align_down;

    if align_down(0, 4) != 0 {
        return TestResult::Fail;
    }
    if align_down(3, 4) != 0 {
        return TestResult::Fail;
    }
    if align_down(4, 4) != 4 {
        return TestResult::Fail;
    }
    if align_down(7, 4) != 4 {
        return TestResult::Fail;
    }
    if align_down(4097, 4096) != 4096 {
        return TestResult::Fail;
    }

    TestResult::Pass
}

fn test_is_aligned() -> TestResult {
    use crate::klib::is_aligned;

    if !is_aligned(0, 4) {
        return TestResult::Fail;
    }
    if !is_aligned(4, 4) {
        return TestResult::Fail;
    }
    if !is_aligned(4096, 4096) {
        return TestResult::Fail;
    }
    if is_aligned(1, 4) {
        return TestResult::Fail;
    }
    if is_aligned(4097, 4096) {
        return TestResult::Fail;
    }

    TestResult::Pass
}

// =============================================================================
// TESTES DE BITS
// =============================================================================

fn test_bit_set_clear() -> TestResult {
    let mut val = 0u64;

    // Setar bit 3
    val |= 1 << 3;
    if (val & (1 << 3)) == 0 {
        return TestResult::Fail;
    }

    // Limpar bit 3
    val &= !(1 << 3);
    if (val & (1 << 3)) != 0 {
        return TestResult::Fail;
    }

    // Setar múltiplos bits
    val |= (1 << 0) | (1 << 7) | (1 << 63);
    if val != (1 | (1 << 7) | (1 << 63)) {
        return TestResult::Fail;
    }

    TestResult::Pass
}
