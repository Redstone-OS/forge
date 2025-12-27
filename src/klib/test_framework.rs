//! # Framework de Self-Tests do Kernel
//!
//! Fornece estruturas e macros para testes padronizados.
//!
//! # Uso
//! ```rust
//! use crate::klib::test_framework::*;
//!
//! pub fn run_my_tests() {
//!     run_test_suite("Meu Módulo", &[
//!         test_something,
//!         test_another,
//!     ]);
//! }
//! ```

/// Resultado de um teste individual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestResult {
    /// Teste passou com sucesso.
    Pass,
    /// Teste falhou com mensagem.
    Fail,
    /// Teste foi pulado (não aplicável no contexto atual).
    Skip,
}

/// Estrutura para um caso de teste.
pub struct TestCase {
    /// Nome do teste (para logging).
    pub name: &'static str,
    /// Função que executa o teste.
    pub func: fn() -> TestResult,
}

impl TestCase {
    /// Cria um novo caso de teste.
    pub const fn new(name: &'static str, func: fn() -> TestResult) -> Self {
        Self { name, func }
    }

    /// Executa o teste e retorna o resultado.
    pub fn run(&self) -> TestResult {
        crate::klog!(crate::core::logging::P_INFO);
        crate::klog!("[Test] ");
        crate::klog!(self.name);
        crate::knl!();

        let result = (self.func)();

        match result {
            TestResult::Pass => {
                crate::klog!(crate::core::logging::P_INFO);
                crate::klog!("[Test] ✓ ");
                crate::klog!(self.name);
                crate::knl!();
            }
            TestResult::Fail => {
                crate::klog!(crate::core::logging::P_ERROR);
                crate::klog!("[Test] ✗ ");
                crate::klog!(self.name);
                crate::knl!();
            }
            TestResult::Skip => {
                crate::klog!(crate::core::logging::P_WARN);
                crate::klog!("[Test] ⊘ ");
                crate::klog!(self.name);
                crate::knl!();
            }
        }
        result
    }
}

/// Executa uma suíte de testes.
///
/// Se algum teste falhar, o kernel entra em panic.
/// Isso garante que o kernel só prossegue se todos os testes passarem.
pub fn run_test_suite(suite_name: &str, tests: &[TestCase]) {
    // Output simplificado - apenas 1 linha de abertura
    crate::klog!(crate::core::logging::P_INFO);
    crate::klog!("🧪 [");
    crate::klog!(suite_name);
    crate::klog!("] Iniciando (");
    crate::drivers::serial::emit_hex(tests.len() as u64);
    crate::klog!(" testes)\n");

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;

    // Usar while para evitar iteradores
    let mut i = 0;
    while i < tests.len() {
        let test = &tests[i];
        match test.run() {
            TestResult::Pass => passed += 1,
            TestResult::Fail => {
                failed += 1;
                // Falha crítica - parar imediatamente
                crate::klog!(crate::core::logging::P_ERROR);
                crate::klog!("SUITE FAILED: ");
                crate::klog!(suite_name);
                crate::knl!();
                panic!("Test suite failed - kernel halted");
            }
            TestResult::Skip => skipped += 1,
        }
        i += 1;
    }

    // Output de sucesso simplificado - apenas 1 linha
    crate::klog!(crate::core::logging::P_INFO);
    crate::klog!("✅ [");
    crate::klog!(suite_name);
    crate::klog!("] ");
    crate::drivers::serial::emit_hex(passed as u64);
    crate::klog!(" OK");
    if skipped > 0 {
        crate::klog!(", ");
        crate::drivers::serial::emit_hex(skipped as u64);
        crate::klog!(" skip");
    }
    crate::klog!("\n");
}

/// Macro para criar asserções em testes.
///
/// Se a condição for falsa, loga erro e retorna Fail.
#[macro_export]
macro_rules! kassert {
    ($cond:expr) => {
        if !($cond) {
            crate::kerror!("ASSERTION FAILED: ", stringify!($cond));
            return $crate::klib::test_framework::TestResult::Fail;
        }
    };
    ($cond:expr, $msg:expr) => {
        if !($cond) {
            crate::kerror!("ASSERTION FAILED: ", $msg);
            return $crate::klib::test_framework::TestResult::Fail;
        }
    };
}

/// Macro para criar asserções de igualdade.
#[macro_export]
macro_rules! kassert_eq {
    ($left:expr, $right:expr) => {
        if ($left) != ($right) {
            crate::kerror!("ASSERTION FAILED: left != right");
            crate::kerror!("  left  = ", $left as u64);
            crate::kerror!("  right = ", $right as u64);
            return $crate::klib::test_framework::TestResult::Fail;
        }
    };
}

/// Macro para definir um teste simples.
///
/// Uso:
/// ```rust
/// ktest!(test_name, {
///     // código do teste
///     kassert!(true);
/// });
/// ```
#[macro_export]
macro_rules! ktest {
    ($name:ident, $body:block) => {
        pub fn $name() -> $crate::klib::test_framework::TestResult {
            $body
            $crate::klib::test_framework::TestResult::Pass
        }
    };
}
