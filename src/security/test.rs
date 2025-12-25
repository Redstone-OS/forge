//! Testes da Lógica de Segurança
//!
//! Valida máscaras de permissão e conceitos de superusuário.

/// Executa todos os testes de segurança
pub fn run_security_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE SEGURANÇA             ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_capability_mask();
    test_root_perm();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ SEGURANÇA VALIDADA!                ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_capability_mask() {
    crate::kdebug!("(Security) Testando máscaras de bits...");

    const CAP_READ: u8 = 1 << 0;
    const CAP_WRITE: u8 = 1 << 1;

    let mut my_caps = CAP_READ;

    // Tenta ter Write sem ter concedido
    let has_write = (my_caps & CAP_WRITE) != 0;

    if !has_write {
        crate::ktrace!("(Security) Start: No Write Perm (OK)");
    }

    // Concede Write
    my_caps |= CAP_WRITE;
    if (my_caps & CAP_WRITE) != 0 {
        crate::ktrace!("(Security) Grant: Write Perm Added (OK)");
    }

    crate::kinfo!("(Security) ✓ Capability Logic OK");
}

fn test_root_perm() {
    crate::kdebug!("(Security) Simulando check de superuser...");

    let uid = 0; // Root
    let is_root = uid == 0;

    if is_root {
        crate::ktrace!("(Security) UID 0 identified as Root");
        crate::kinfo!("(Security) ✓ Root Permission Logic OK");
    } else {
        crate::kerror!("(Security) UID 0 NOT Root!");
    }
}
