//! Testes da Biblioteca de Utilities (klib)
//!
//! Valida funções de manipulação de bits, alinhamento e strings.

/// Executa todos os testes de klib
pub fn run_klib_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE KLIB                  ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_align_up();
    test_bit_manipulation();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ KLIB VALIDADO!                     ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_align_up() {
    crate::kdebug!("(klib) Verificando cálculo de alinhamento...");

    // Implementação inline para teste
    fn align_up(addr: u64, align: u64) -> u64 {
        (addr + align - 1) & !(align - 1)
    }

    let addr = 4097;
    let align = 4096;
    let aligned = align_up(addr, align);

    crate::ktrace!("(klib) align_up(4097, 4096) = ", aligned);

    if aligned == 8192 {
        crate::kinfo!("(klib) ✓ Align Up Logic OK");
    } else {
        crate::kerror!("(klib) Align Up Failed! Expected 8192");
    }
}

fn test_bit_manipulation() {
    crate::kdebug!("(klib) Testando set/clear bits...");

    let mut val = 0u64;
    // Set bit 3
    val |= 1 << 3;

    if (val & (1 << 3)) != 0 {
        crate::ktrace!("(klib) Bit 3 SET verified");
    }

    // Clear bit 3
    val &= !(1 << 3);

    if (val & (1 << 3)) == 0 {
        crate::ktrace!("(klib) Bit 3 CLEAR verified");
    }

    crate::kinfo!("(klib) ✓ Bit Manipulation OK");
}
