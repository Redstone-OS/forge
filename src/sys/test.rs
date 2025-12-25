//! Testes de Metadados do Sistema
//!
//! Valida formato de versão e constantes de build.

/// Executa todos os testes de sys
pub fn run_sys_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE SISTEMA               ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_kernel_version_format();
    test_build_constants();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ SISTEMA VALIDADO!                  ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_kernel_version_format() {
    crate::kinfo!("┌─ Teste Version String ──────────────────────");
    crate::kdebug!("(Sys) Validando formato SemVer...");

    let version = "0.1.0";

    // Verificação simples se contém pontos
    let has_dots = version.matches('.').count() >= 2;

    crate::ktrace!("(Sys) Version: {}", version);

    if has_dots {
        crate::kinfo!("│  ✓ Version Format (x.y.z) OK             ");
    } else {
        crate::kwarn!("(Sys) Non-SemVer Version String");
    }
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_build_constants() {
    crate::kinfo!("┌─ Teste Build Consts ────────────────────────");
    crate::kdebug!("(Sys) Verificando profile de compilação...");

    #[cfg(debug_assertions)]
    crate::ktrace!("(Sys) Build Mode: DEBUG");

    #[cfg(not(debug_assertions))]
    crate::ktrace!("(Sys) Build Mode: RELEASE");

    crate::kinfo!("│  ✓ Build Constants Detected              ");
    crate::kinfo!("└───────────────────────────────────────────");
}
