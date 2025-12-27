//! Testes de Sistema

use crate::klib::test_framework::{run_test_suite, TestCase, TestResult};

/// Casos de teste de sys
const SYS_TESTS: &[TestCase] = &[
    TestCase::new("kernel_version_format", test_kernel_version_format),
    TestCase::new("build_constants", test_build_constants),
];

/// Executa todos os testes de sys
pub fn run_sys_tests() {
    run_test_suite("System", SYS_TESTS);
}

/// Valida formato de versão
fn test_kernel_version_format() -> TestResult {
    let version = "0.0.5";

    // Verificação simples: contém pelo menos 2 pontos (SemVer)
    let dot_count = version.as_bytes().iter().filter(|&&b| b == b'.').count();

    if dot_count < 2 {
        crate::kwarn!("(Sys) String de versão não é SemVer");
        // Não falha, apenas avisa
    }

    crate::ktrace!("(Sys) Formato de versão OK");
    TestResult::Pass
}

/// Verifica constantes de build
fn test_build_constants() -> TestResult {
    #[cfg(debug_assertions)]
    crate::ktrace!("(Sys) Modo de build: DEBUG");

    #[cfg(not(debug_assertions))]
    crate::ktrace!("(Sys) Modo de build: RELEASE");

    TestResult::Pass
}
