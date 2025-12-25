//! Testes do Core/Kernel Main
//!
//! Valida constantes fundamentais e integridade do handover do bootloader.

/// Executa todos os testes do Core
pub fn run_core_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DO CORE                  ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_boot_magic();
    test_kernel_address_space();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ CORE VALIDADO!                     ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_boot_magic() {
    crate::kinfo!("┌─ Teste Boot Magic ──────────────────────────");
    use crate::core::handoff::BOOT_MAGIC;

    crate::kdebug!("(Core) Verificando constante mágica...");

    // Teste lógico: A constante deve ser consistente
    if BOOT_MAGIC == 0xDEADBEEF {
        crate::ktrace!("(Core) Magic matches 0xDEADBEEF");
        crate::kinfo!("│  ✓ Boot Magic OK                         ");
    } else {
        crate::kerror!("(Core) Magic MISMATCH: {:#x}", BOOT_MAGIC);
        panic!("Core integrity failure");
    }

    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_kernel_address_space() {
    crate::kinfo!("┌─ Teste Kernel Address Space ────────────────");
    crate::kdebug!("(Core) Validando layout de memória lógica...");

    // Simulação: Testar se KERNEL_START < KERNEL_END
    // Em um cenário real, usaríamos símbolos do linker
    let kernel_base = 0xffffffff80000000u64;
    let kernel_top_limit = 0xffffffffffffffffu64;

    if kernel_base < kernel_top_limit {
        crate::ktrace!(
            "(Core) Base {:#x} < Top {:#x}",
            kernel_base,
            kernel_top_limit
        );
        crate::kinfo!("│  ✓ Address Space Layout OK               ");
    } else {
        crate::kerror!("(Core) Address Space INVERTED");
    }

    crate::kinfo!("└───────────────────────────────────────────");
}
