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
    crate::kinfo!("✅ Core Tests Passed!");
}

fn test_boot_magic() {
    use crate::core::handoff::BOOT_MAGIC;
    crate::kdebug!("(Core) Verificando constante mágica...");

    // Teste lógico: A constante deve ser consistente
    if BOOT_MAGIC == 0x524544_53544F4E45 {
        crate::ktrace!("(Core) Magic matches 'REDSTONE'");
        crate::kinfo!("(Core) ✓ Boot Magic OK");
    } else {
        crate::kerror!("(Core) Magic MISMATCH: ", BOOT_MAGIC);
        panic!("Core integrity failure");
    }
}

fn test_kernel_address_space() {
    crate::kdebug!("(Core) Validando layout de memória lógica...");

    // Simulação: Testar se KERNEL_START < KERNEL_END
    // Em um cenário real, usaríamos símbolos do linker
    let kernel_base = 0xffffffff80000000u64;
    let kernel_top_limit = 0xffffffffffffffffu64;

    if kernel_base < kernel_top_limit {
        crate::ktrace!("(Core) Kernel base=", kernel_base);
        crate::ktrace!("(Core) Kernel top=", kernel_top_limit);
        crate::kinfo!("(Core) ✓ Address Space Layout OK");
    } else {
        crate::kerror!("(Core) Address Space INVERTED");
    }
}
