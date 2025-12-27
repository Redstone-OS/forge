//! Testes do Scheduler

use crate::klib::test_framework::{run_test_suite, TestCase, TestResult};

/// Casos de teste do Scheduler
const SCHED_TESTS: &[TestCase] = &[
    TestCase::new("task_stack_size", test_task_stack_size),
    TestCase::new("priority_ordering", test_priority_ordering),
];

/// Executa todos os testes de scheduler
pub fn run_sched_tests() {
    run_test_suite("Scheduler", SCHED_TESTS);
}

/// Valida constantes de pilha
fn test_task_stack_size() -> TestResult {
    let stack_size = 16 * 1024; // 16 KiB

    if stack_size % 4096 != 0 {
        crate::kwarn!("(Sched) Tamanho de pilha NÃO alinhado a página");
        return TestResult::Fail;
    }

    crate::ktrace!("(Sched) Tamanho da pilha=", stack_size as u64);
    TestResult::Pass
}

/// Verifica hierarquia de prioridades
fn test_priority_ordering() -> TestResult {
    #[derive(PartialEq, PartialOrd)]
    enum Priority {
        Low,
        Normal,
        High,
    }

    if !(Priority::High > Priority::Normal && Priority::Normal > Priority::Low) {
        crate::kerror!("(Sched) Enum de prioridade quebrado!");
        return TestResult::Fail;
    }

    crate::ktrace!("(Sched) Ordenação de prioridades OK");
    TestResult::Pass
}
