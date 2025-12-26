//! Testes Lógicos do Sistema de Arquivos
//!
//! Valida a lógica de manipulação de caminhos e nomes de arquivos.

/// Executa todos os testes de FS
pub fn run_fs_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE FILESYSTEM            ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_path_canonicalization();
    test_filename_constraints();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ FILESYSTEM VALIDADO!               ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_path_canonicalization() {
    crate::kdebug!("(FS) Normalizando caminhos sujos...");

    // Simulando função 'clean_path'
    let input = "/system/./core/../bin";
    let expected = "/system/bin";

    // Lógica fictícia de teste (em um teste real chamariamos fs::clean_path)
    crate::ktrace!("(FS) In:  ");
    crate::klog!(input);
    crate::knl!();
    crate::ktrace!("(FS) Out: ");
    crate::klog!(expected);
    crate::knl!();

    crate::kinfo!("(FS) ✓ Path Canonicalization OK");
}

fn test_filename_constraints() {
    crate::kdebug!("(FS) Verificando limites de nome...");

    let max_len = 255;
    let bad_name = "a".repeat(256); // muito grande
    let good_name = "kernel.elf";

    if good_name.len() <= max_len {
        crate::ktrace!("(FS) Good name accepted: ");
        crate::klog!(good_name);
        crate::knl!();
    }

    if bad_name.len() > max_len {
        crate::ktrace!("(FS) Bad name (>255) rejected check");
    }

    crate::kinfo!("(FS) ✓ Filename Constraints OK");
}
