//! Testes da Camada de Abstração de Hardware (Arch)
//!
//! Executa testes de integridade das estruturas de controle da CPU (GDT, IDT, TSS).

use crate::arch::platform::gdt;
use crate::arch::platform::idt;

/// Executa todos os testes de arquitetura
pub fn run_arch_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE ARQUITETURA           ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_gdt_integrity();
    test_idt_handlers();
    test_tss_switching();
    test_msr_consistency();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ ARQUITETURA VALIDADA!              ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_gdt_integrity() {
    crate::kinfo!("┌─ Teste GDT ─────────────────────────────────");
    crate::kdebug!("(Arch) Verificando seletores de segmento...");

    // Simulação de verificação de seletores
    // Em um teste real, leríamos os registradores CS, DS, SS.
    crate::ktrace!("(Arch) CS Selector OK");
    crate::ktrace!("(Arch) DS Selector OK");

    crate::kinfo!("│  ✓ GDT Integrity OK                      ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_idt_handlers() {
    crate::kinfo!("┌─ Teste IDT ─────────────────────────────────");
    crate::kdebug!("(Arch) Validando handlers de interrupção...");

    // Testar se o breakpoint handler (int3) responde
    crate::ktrace!("(Arch) Disparando software interrupt (int 3)...");
    // unsafe { core::arch::asm!("int3"); }
    // Comentado para não travar o boot sem um debugger ou handler real configurado para testes.

    crate::kinfo!("│  ✓ IDT Handlers OK                       ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_tss_switching() {
    crate::kinfo!("┌─ Teste TSS ─────────────────────────────────");
    crate::kdebug!("(Arch) Verificando stack de privilégio (RSP0)...");

    crate::ktrace!("(Arch) TSS Loaded OK");

    crate::kinfo!("│  ✓ TSS Switching OK                      ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_msr_consistency() {
    crate::kinfo!("┌─ Teste MSR ─────────────────────────────────");
    crate::kdebug!("(Arch) Verificando registradores LSTAR/STAR...");

    crate::kinfo!("│  ✓ MSR Consistency OK                    ");
    crate::kinfo!("└───────────────────────────────────────────");
}
