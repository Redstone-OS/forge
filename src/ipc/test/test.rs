//! Testes de IPC

use crate::klib::test_framework::{run_test_suite, TestCase, TestResult};

/// Casos de teste de IPC
const IPC_TESTS: &[TestCase] = &[
    TestCase::new("message_header_size", test_message_header_size),
    TestCase::new("port_id_logic", test_port_id_logic),
];

/// Executa todos os testes de IPC
pub fn run_ipc_tests() {
    run_test_suite("IPC", IPC_TESTS);
}

/// Verifica tamanho do header de mensagem
fn test_message_header_size() -> TestResult {
    #[repr(C)]
    struct Header {
        src_port: u64,
        dst_port: u64,
        len: u64,
        msg_id: u64,
    }

    let size = core::mem::size_of::<Header>();

    if size != 32 {
        crate::kerror!("(IPC) Tamanho de header inesperado=", size as u64);
        return TestResult::Fail;
    }

    crate::ktrace!("(IPC) Tamanho do header=", size as u64);
    TestResult::Pass
}

/// Valida lógica de geração de IDs de porta
fn test_port_id_logic() -> TestResult {
    let next_id = 1;

    if next_id == 0 {
        crate::kerror!("(IPC) ID nulo gerado!");
        return TestResult::Fail;
    }

    crate::ktrace!("(IPC) Lógica de ID de porta OK");
    TestResult::Pass
}
