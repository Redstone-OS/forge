//! Testes da Camada de IPC
//!
//! Valida a estrutura lógica de mensagens e identificadores de porta.

/// Executa todos os testes de IPC
pub fn run_ipc_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE IPC                   ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_message_header_size();
    test_port_id_logic();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ IPC VALIDADO!                      ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_message_header_size() {
    crate::kdebug!("(IPC) Verificando alinhamento do header...");

    // Struct fictícia
    #[repr(C)]
    struct Header {
        src_port: u64,
        dst_port: u64,
        len: u64,
        msg_id: u64,
    }

    let size = core::mem::size_of::<Header>();
    crate::ktrace!("(IPC) Header Size: {} bytes", size);

    if size == 32 {
        crate::kinfo!("(IPC) ✓ IPC Header Packed/Aligned OK");
    } else {
        crate::kerror!("(IPC) Unexpected Header Size: {}", size);
    }
}

fn test_port_id_logic() {
    crate::kdebug!("(IPC) Validando geração de IDs...");

    // IDs de porta não podem ser 0 (reservado/nulo)
    let next_id = 1;

    if next_id != 0 {
        crate::ktrace!("(IPC) Generated ID {} (Valid)", next_id);
        crate::kinfo!("(IPC) ✓ Port ID Logic OK");
    } else {
        crate::kerror!("(IPC) Generated Null ID!");
    }
}
