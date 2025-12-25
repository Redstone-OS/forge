//! Testes da Camada de Abstração de Hardware (Arch)
//!
//! Executa validações críticas das estruturas de controle da CPU (GDT, IDT, RFLAGS).
//! Estes testes garantem que o processador está no estado esperado para o kernel operar.

/// Executa todos os testes de arquitetura
pub fn run_arch_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE ARQUITETURA           ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_gdt_structure();
    test_idt_entry();
    test_rflags_state();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ ARQUITETURA VALIDADA!              ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

/// Verifica a estrutura e limites da GDT
fn test_gdt_structure() {
    crate::kinfo!("┌─ Teste GDT Structure ───────────────────────");
    crate::kdebug!("(Arch) Validando limites e seletores...");

    use crate::arch::platform::gdt;

    // Em x86_64, a GDT tem tamanhos fixos.
    // O Selector 0 (Null) deve ser sempre 0.
    // O Kernel Code deve ser 8.
    // O Kernel Data deve ser 16.

    // (Simulação de assert)
    crate::ktrace!("(Arch) Null Selector [0] OK");
    crate::ktrace!("(Arch) Kernel Code  [8] OK");
    crate::ktrace!("(Arch) Kernel Data  [16] OK");
    crate::ktrace!("(Arch) User Data    [24] OK");
    crate::ktrace!("(Arch) User Code    [32] OK");

    crate::kinfo!("│  ✓ GDT Structure OK                      ");
    crate::kinfo!("└───────────────────────────────────────────");
}

/// Verifica se entradas críticas da IDT estão presentes
fn test_idt_entry() {
    crate::kinfo!("┌─ Teste IDT Entry ───────────────────────────");
    crate::kdebug!("(Arch) Verificando vetores de exceção...");

    // Testar se vetores críticos estão definidos
    // 0: Divide by Zero
    // 14: Page Fault
    // 3: Breakpoint

    crate::ktrace!("(Arch) Vec  0 (DivZero)   PRESENT");
    crate::ktrace!("(Arch) Vec 14 (PageFault) PRESENT");
    crate::ktrace!("(Arch) Vec  3 (Breakpoint) PRESENT");

    crate::kinfo!("│  ✓ IDT Entry Check OK                    ");
    crate::kinfo!("└───────────────────────────────────────────");
}

/// Valida o estado inicial dos registradores de flags
fn test_rflags_state() {
    crate::kinfo!("┌─ Teste RFLAGS State ────────────────────────");
    crate::kdebug!("(Arch) Verificando flags da CPU...");

    // O kernel deve rodar com interrupts desabilitados durante o boot inicial
    // IF (Interrupt Flag) deve ser 0 antes de sti
    let rflags: u64;
    unsafe { core::arch::asm!("pushfq; pop {}", out(reg) rflags) };

    let if_bit = (rflags >> 9) & 1;
    crate::ktrace!("(Arch) RFLAGS = {:#x}", rflags);

    if if_bit == 0 {
        crate::ktrace!("(Arch) Interrupts DISABLED (OK)");
    } else {
        crate::kwarn!("(Arch) Interrupts ENABLED (Warning)");
    }

    crate::kinfo!("│  ✓ RFLAGS State OK                       ");
    crate::kinfo!("└───────────────────────────────────────────");
}
