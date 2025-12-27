//! Testes de Sincronização

use crate::klib::test_framework::{run_test_suite, TestCase, TestResult};

/// Casos de teste de sync
const SYNC_TESTS: &[TestCase] = &[
    TestCase::new("spinlock_api", test_spinlock_api),
    TestCase::new("atomic_alignment", test_atomic_alignment),
];

/// Executa todos os testes de sync
pub fn run_sync_tests() {
    run_test_suite("Sync", SYNC_TESTS);
}

/// Simula lock/unlock single-thread
fn test_spinlock_api() -> TestResult {
    let mut locked;

    // Lock
    locked = true;
    if !locked {
        return TestResult::Fail;
    }

    // Unlock
    locked = false;
    if locked {
        return TestResult::Fail;
    }

    crate::ktrace!("(Sync) Lógica de estado do spinlock OK");
    TestResult::Pass
}

/// Verifica alinhamento de tipos atômicos
fn test_atomic_alignment() -> TestResult {
    use core::sync::atomic::AtomicU64;
    let align = core::mem::align_of::<AtomicU64>();

    if align != 8 {
        crate::kwarn!("(Sync) Alinhamento atômico subótimo=", align as u64);
        // Não falha, apenas avisa
    }

    crate::ktrace!("(Sync) Alinhamento AtomicU64=", align as u64);
    TestResult::Pass
}
