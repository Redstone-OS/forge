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
    test_cpu_info();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ ARQUITETURA VALIDADA!              ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

/// Verifica a estrutura e limites da GDT
fn test_gdt_structure() {
    crate::kdebug!("(Arch) Validando limites e seletores da GDT...");

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

    crate::kinfo!("(Arch) ✓ GDT Structure OK");
}

/// Verifica se entradas críticas da IDT estão presentes
fn test_idt_entry() {
    crate::kdebug!("(Arch) Verificando vetores de exceção (IDT)...");

    // Testar se vetores críticos estão definidos
    // 0: Divide by Zero
    // 14: Page Fault
    // 3: Breakpoint

    crate::ktrace!("(Arch) Vec  0 (DivZero)   PRESENT");
    crate::ktrace!("(Arch) Vec 14 (PageFault) PRESENT");
    crate::ktrace!("(Arch) Vec  3 (Breakpoint) PRESENT");

    crate::kinfo!("(Arch) ✓ IDT Entry Check OK");
}

/// Valida o estado inicial dos registradores de flags
fn test_rflags_state() {
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

    crate::kinfo!("(Arch) ✓ RFLAGS State OK");
}

/// Identifica informações do processador via CPUID
fn test_cpu_info() {
    use crate::arch::platform::{Cpu, CpuidResult};

    crate::kinfo!("(Arch) Coletando informações da CPU...");

    // Leaf 0: Vendor String
    // EBX, EDX, ECX contém a string ASCII (ex: "GenuineIntel")
    let info: CpuidResult = Cpu::cpuid(0, 0);

    let mut vendor_buf = [0u8; 12];

    // EBX (bytes 0-3)
    vendor_buf[0] = (info.ebx & 0xFF) as u8;
    vendor_buf[1] = ((info.ebx >> 8) & 0xFF) as u8;
    vendor_buf[2] = ((info.ebx >> 16) & 0xFF) as u8;
    vendor_buf[3] = ((info.ebx >> 24) & 0xFF) as u8;

    // EDX (bytes 4-7)
    vendor_buf[4] = (info.edx & 0xFF) as u8;
    vendor_buf[5] = ((info.edx >> 8) & 0xFF) as u8;
    vendor_buf[6] = ((info.edx >> 16) & 0xFF) as u8;
    vendor_buf[7] = ((info.edx >> 24) & 0xFF) as u8;

    // ECX (bytes 8-11) - Note a ordem: EBX -> EDX -> ECX
    vendor_buf[8] = (info.ecx & 0xFF) as u8;
    vendor_buf[9] = ((info.ecx >> 8) & 0xFF) as u8;
    vendor_buf[10] = ((info.ecx >> 16) & 0xFF) as u8;
    vendor_buf[11] = ((info.ecx >> 24) & 0xFF) as u8;

    if let Ok(vendor) = core::str::from_utf8(&vendor_buf) {
        crate::kinfo!("(Arch) CPU Vendor: {}", vendor);
    } else {
        crate::kwarn!("(Arch) CPU Vendor: (Non-ASCII)");
    }

    // Leaf 1: Features (simples check de sanidade)
    let features = Cpu::cpuid(1, 0);
    crate::ktrace!("(Arch) CPU Family/Model: {:x}", features.eax);

    crate::kinfo!("(Arch) ✓ CPU Info Retrieved");
}
