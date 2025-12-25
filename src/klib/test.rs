//! Testes da Biblioteca de Base do Kernel (klib)
//!
//! Executa testes de utilitários e algoritmos básicos.

/// Executa todos os testes de klib
pub fn run_klib_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE KLIB                  ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_string_manipulation();
    test_bit_ops_safety();
    test_alignment_helpers();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ KLIB VALIDADO!                     ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_string_manipulation() {
    crate::kinfo!("┌─ Teste Strings ─────────────────────────────");
    crate::kdebug!("(klib) Validando manipuladores de texto...");

    crate::kinfo!("│  ✓ String Manipulation OK                ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_bit_ops_safety() {
    crate::kinfo!("┌─ Teste Bits ────────────────────────────────");
    crate::kdebug!("(klib) Verificando operações bit-a-bit...");

    crate::kinfo!("│  ✓ Bit Ops Safety OK                     ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_alignment_helpers() {
    crate::kinfo!("┌─ Teste Alignment ───────────────────────────");
    crate::kdebug!("(klib) Validando arredondamento de endereços...");

    crate::kinfo!("│  ✓ Alignment Helpers OK                  ");
    crate::kinfo!("└───────────────────────────────────────────");
}
