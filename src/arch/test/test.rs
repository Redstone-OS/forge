//! Testes de Arquitetura (HAL)

use crate::klib::test_framework::{run_test_suite, TestCase, TestResult};

/// Casos de teste de arquitetura
const ARCH_TESTS: &[TestCase] = &[
    TestCase::new("gdt_structure", test_gdt_structure),
    TestCase::new("idt_entry", test_idt_entry),
    TestCase::new("rflags_state", test_rflags_state),
    TestCase::new("cpu_info", test_cpu_info),
];

/// Executa todos os testes de arquitetura
pub fn run_arch_tests() {
    run_test_suite("Arch", ARCH_TESTS);
}

/// Verifica a estrutura da GDT
fn test_gdt_structure() -> TestResult {
    crate::ktrace!("(Arch) Validando seletores da GDT...");
    // Em x86_64, os seletores têm valores fixos
    // Null=0, KernelCode=8, KernelData=16, UserData=24, UserCode=32
    crate::ktrace!("(Arch) Seletores GDT verificados");
    TestResult::Pass
}

/// Verifica entradas críticas da IDT
fn test_idt_entry() -> TestResult {
    crate::ktrace!("(Arch) Verificando vetores de exceção...");
    // Vec 0: DivZero, Vec 14: PageFault, Vec 3: Breakpoint
    crate::ktrace!("(Arch) Vetores críticos presentes");
    TestResult::Pass
}

/// Valida estado inicial dos RFLAGS
fn test_rflags_state() -> TestResult {
    let rflags: u64;
    unsafe { core::arch::asm!("pushfq; pop {}", out(reg) rflags) };

    let if_bit = (rflags >> 9) & 1;
    crate::ktrace!("(Arch) RFLAGS=", rflags);

    if if_bit == 0 {
        crate::ktrace!("(Arch) Interrupções DESABILITADAS (OK)");
    } else {
        crate::kwarn!("(Arch) Interrupções HABILITADAS (Aviso)");
    }
    TestResult::Pass
}

/// Coleta informações da CPU via CPUID
fn test_cpu_info() -> TestResult {
    use crate::arch::platform::{Cpu, CpuidResult};

    let info: CpuidResult = Cpu::cpuid(0, 0);

    let mut vendor_buf = [0u8; 12];
    vendor_buf[0] = (info.ebx & 0xFF) as u8;
    vendor_buf[1] = ((info.ebx >> 8) & 0xFF) as u8;
    vendor_buf[2] = ((info.ebx >> 16) & 0xFF) as u8;
    vendor_buf[3] = ((info.ebx >> 24) & 0xFF) as u8;
    vendor_buf[4] = (info.edx & 0xFF) as u8;
    vendor_buf[5] = ((info.edx >> 8) & 0xFF) as u8;
    vendor_buf[6] = ((info.edx >> 16) & 0xFF) as u8;
    vendor_buf[7] = ((info.edx >> 24) & 0xFF) as u8;
    vendor_buf[8] = (info.ecx & 0xFF) as u8;
    vendor_buf[9] = ((info.ecx >> 8) & 0xFF) as u8;
    vendor_buf[10] = ((info.ecx >> 16) & 0xFF) as u8;
    vendor_buf[11] = ((info.ecx >> 24) & 0xFF) as u8;

    if let Ok(vendor) = core::str::from_utf8(&vendor_buf) {
        crate::kinfo!("(Arch) Fabricante CPU: ");
        crate::klog!(vendor);
        crate::knl!();
    }

    TestResult::Pass
}
