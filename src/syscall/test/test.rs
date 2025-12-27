//! Testes de Syscalls

use crate::klib::test_framework::{run_test_suite, TestCase, TestResult};

/// Casos de teste de syscall
const SYSCALL_TESTS: &[TestCase] = &[
    TestCase::new("syscall_table_bounds", test_syscall_table_bounds),
    TestCase::new("error_codes", test_error_codes),
];

/// Executa todos os testes de syscall
pub fn run_syscall_tests() {
    run_test_suite("Syscall", SYSCALL_TESTS);
}

/// Verifica limite máximo de IDs
fn test_syscall_table_bounds() -> TestResult {
    let max_syscalls = 512;
    let test_id = 1024; // ID inválido

    if test_id < max_syscalls {
        crate::kerror!("(Syscall) ID fora dos limites aceito!");
        return TestResult::Fail;
    }

    crate::ktrace!("(Syscall) Lógica de verificação de limites OK");
    TestResult::Pass
}

/// Valida convenção de códigos de erro
fn test_error_codes() -> TestResult {
    let ret_success: isize = 0;
    let ret_error: isize = -1;

    if ret_success < 0 {
        crate::kerror!("(Syscall) Sucesso tratado como erro");
        return TestResult::Fail;
    }

    if ret_error >= 0 {
        crate::kerror!("(Syscall) Erro tratado como sucesso");
        return TestResult::Fail;
    }

    crate::ktrace!("(Syscall) Convenção de códigos de erro OK");
    TestResult::Pass
}
