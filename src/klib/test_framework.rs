//! Framework de testes do kernel

/// Resultado de teste
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestResult {
    Passed,
    Failed,
    Skipped,
}

/// Um caso de teste
pub struct TestCase {
    pub name: &'static str,
    pub func: fn() -> TestResult,
}

/// Executa suite de testes
pub fn run_test_suite(name: &str, tests: &[TestCase]) -> (usize, usize, usize) {
    // Nota: Imprime o endereço da string do nome porque o klog atual não suporta %s
    crate::kinfo!("=== Executando suite:", name.as_ptr() as u64);
    
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    
    for test in tests {
        let result = (test.func)();
        match result {
            TestResult::Passed => {
                crate::kinfo!("[PASS]", test.name.as_ptr() as u64);
                passed += 1;
            }
            TestResult::Failed => {
                crate::kerror!("[FAIL]", test.name.as_ptr() as u64);
                failed += 1;
            }
            TestResult::Skipped => {
                crate::kwarn!("[SKIP]", test.name.as_ptr() as u64);
                skipped += 1;
            }
        }
    }
    
    crate::kinfo!("Resultados: passed=", passed as u64);
    (passed, failed, skipped)
}
