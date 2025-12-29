/// Arquivo: x86_64/interrupts.rs
///
/// Propósito: Definição e registro de Handlers de Interrupção.
/// Configura a IDT com handlers para exceções da CPU (Divide Error, Page Fault, GPF, etc.)
/// e mapeia IRQs de hardware.
///
/// Detalhes de Implementação:
/// - Inicializa a IDT global.
/// - Define stubs/handlers para as exceções críticas.
/// - Implementa `init_idt`.
// Handlers de interrupção e inicialização da IDT
use crate::arch::x86_64::idt::IDT;

/// Stack Frame pushed by CPU on exception
#[repr(C)]
#[derive(Debug)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

/// Inicializa a Tabela de Descritores de Interrupção (IDT).
/// Deve ser chamado no boot antes de habilitar interrupções.
pub fn init_idt() {
    // SAFETY: Acesso à estática mutável IDT é seguro aqui pois estamos no boot single-core
    unsafe {
        // Registrar handlers de exceção básicos
        IDT.set_handler(0, divide_error_handler as u64);
        IDT.set_handler(3, breakpoint_handler as u64);
        IDT.set_handler(6, invalid_opcode_handler as u64);
        IDT.set_handler(8, double_fault_handler as u64);
        IDT.set_handler(13, general_protection_handler as u64);
        IDT.set_handler(14, page_fault_handler as u64);

        // Registrar handlers de IRQ (32-47)
        // Por enquanto, stub genérico
        // TODO: IRQ Router
        for i in 32..48 {
            IDT.set_handler(i, irq_handler_stub as u64);
        }

        // Carregar IDT
        IDT.load();
    }
}

// =============================================================================
// HANDLERS
// =============================================================================

extern "x86-interrupt" fn divide_error_handler(stack_frame: ExceptionStackFrame) {
    crate::kerror!("EXCEPTION: DIVIDE ERROR (#DE)");
    crate::kerror!("RIP: {:x}", stack_frame.instruction_pointer);
    crate::kerror!("RSP: {:x}", stack_frame.stack_pointer);
    loop {
        crate::arch::Cpu::halt();
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: ExceptionStackFrame) {
    crate::kinfo!(
        "EXCEPTION: BREAKPOINT at {:x}",
        stack_frame.instruction_pointer
    );
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: ExceptionStackFrame) {
    crate::kerror!("EXCEPTION: INVALID OPCODE (#UD)");
    crate::kerror!("RIP: {:x}", stack_frame.instruction_pointer);
    loop {
        crate::arch::Cpu::halt();
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: ExceptionStackFrame,
    _error_code: u64,
) -> ! {
    crate::kerror!("EXCEPTION: DOUBLE FAULT (#DF)");
    crate::kerror!("RIP: {:x}", stack_frame.instruction_pointer);
    panic!("DOUBLE FAULT");
}

extern "x86-interrupt" fn general_protection_handler(
    stack_frame: ExceptionStackFrame,
    error_code: u64,
) {
    crate::kerror!("EXCEPTION: GENERAL PROTECTION FAULT (#GP)");
    crate::kerror!("RIP: {:x}", stack_frame.instruction_pointer);
    crate::kerror!("Error Code: {:x}", error_code);
    loop {
        crate::arch::Cpu::halt();
    }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: ExceptionStackFrame, error_code: u64) {
    let cr2: u64;
    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
    }

    crate::kerror!("EXCEPTION: PAGE FAULT (#PF)");
    crate::kerror!("Accessed Address: {:x}", cr2);
    crate::kerror!("Error Code: {:x}", error_code);
    crate::kerror!("RIP: {:x}", stack_frame.instruction_pointer);
    crate::kerror!("CS: {:x}", stack_frame.code_segment);
    crate::kerror!("RFLAGS: {:x}", stack_frame.cpu_flags);
    crate::kerror!("RSP: {:x}", stack_frame.stack_pointer);
    crate::kerror!("SS: {:x}", stack_frame.stack_segment);

    loop {
        crate::arch::Cpu::halt();
    }
}

extern "x86-interrupt" fn irq_handler_stub(_stack_frame: ExceptionStackFrame) {
    // Acknowledge interrupt (EOI) if APIC needed, but for now just loop/print
    // crate::kinfo!("IRQ Stub Called");
    // TODO: Send EOI to LAPIC
    // E.g. crate::arch::x86_64::apic::lapic::end_of_interrupt();
    // But we don't have safe access here yet.
    // Just ignore for now or disable interrupt?
}
