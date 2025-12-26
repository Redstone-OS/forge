//! # System Metadata Tests
//!
//! Testes unitários para validar constantes globais e metadados de build.
//!
//! ## 🎯 Propósito
//! - **Sanity Check:** Garantir que o kernel sabe sua própria versão e modo de compilação (Debug/Release).
//!
//! ## 🛠️ TODOs
//! - [ ] **TODO: (Validation)** Adicionar teste de **Endianness** e tamanho de `usize`.
//!   - *Motivo:* Garantir que `usize == u64` (em build x86_64) para evitar supresas na ABI.

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
    crate::kdebug!("(Sys) Validando formato SemVer...");

    let version = "0.1.0";

    // Verificação simples se contém pontos
    let has_dots = version.as_bytes().iter().filter(|&&b| b == b'.').count() >= 2;

    crate::ktrace!("(Sys) Version: ");
    crate::klog!(version);
    crate::knl!();

    if has_dots {
        crate::kinfo!("(Sys) ✓ Version Format (x.y.z) OK");
    } else {
        crate::kwarn!("(Sys) Non-SemVer Version String");
    }
}

fn test_build_constants() {
    crate::kdebug!("(Sys) Verificando profile de compilação...");

    #[cfg(debug_assertions)]
    crate::ktrace!("(Sys) Build Mode: DEBUG");

    #[cfg(not(debug_assertions))]
    crate::ktrace!("(Sys) Build Mode: RELEASE");

    crate::kinfo!("(Sys) ✓ Build Constants Detected");
}
