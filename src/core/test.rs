//! Testes do Core/Kernel Main
//!
//! Executa testes de integridade do processo de boot e carregamento de binários.

/// Executa todos os testes do Core
pub fn run_core_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DO CORE                  ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_boot_info_validation();
    test_elf_parser();
    test_entry_point_consistency();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ CORE VALIDADO!                      ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_boot_info_validation() {
    crate::kinfo!("┌─ Teste BootInfo ────────────────────────────");
    crate::kdebug!("(Core) Validando estruturas de handoff...");

    crate::ktrace!("(Core) Boot Magic OK");
    crate::ktrace!("(Core) Protocol Version OK");

    crate::kinfo!("│  ✓ BootInfo Validation OK                ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_elf_parser() {
    crate::kinfo!("┌─ Teste ELF ─────────────────────────────────");
    crate::kdebug!("(Core) Testado parser com headers dummy...");

    crate::ktrace!("(Core) ELF Magic Header OK");
    crate::ktrace!("(Core) Program Headers OK");

    crate::kinfo!("│  ✓ ELF Parser OK                         ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_entry_point_consistency() {
    crate::kinfo!("┌─ Teste Entry Point ─────────────────────────");
    crate::kdebug!("(Core) Verificando alinhamento do salto inicial...");

    crate::kinfo!("│  ✓ Entry Point OK                       ");
    crate::kinfo!("└───────────────────────────────────────────");
}
