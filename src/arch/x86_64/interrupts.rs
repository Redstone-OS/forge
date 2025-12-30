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
    let idt = unsafe { &mut *core::ptr::addr_of_mut!(IDT) };

    // Debug: Print handler address
    crate::kinfo!(
        "(IDT) Divide Error Handler Addr:",
        divide_error_handler as u64
    );

    idt.set_handler(0, divide_error_handler as u64);
    idt.set_handler(3, breakpoint_handler as u64);
    idt.set_handler(6, invalid_opcode_handler as u64);
    idt.set_handler(8, double_fault_handler as u64);
    idt.set_handler(13, general_protection_handler as u64);
    idt.set_handler(14, page_fault_handler as u64);

    // Remapear IRQs (PIC) -> 32..47
    // Timer (IRQ 0) -> 32
    // Agora usamos o handler asm 'timer_handler' para permitir preempção
    idt.set_handler(32, timer_handler as u64);
    idt.set_handler(33, keyboard_interrupt_handler as u64);
    idt.set_handler(44, mouse_interrupt_handler as u64);

    unsafe {
        idt.load();
    }
}

// =============================================================================
// HANDLERS ASM
// =============================================================================

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: ExceptionStackFrame) {
    crate::drivers::input::keyboard::handle_irq();
    crate::arch::x86_64::ports::outb(0x20, 0x20); // EOI Master
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: ExceptionStackFrame) {
    crate::drivers::input::mouse::handle_irq();
    crate::arch::x86_64::ports::outb(0xA0, 0x20); // EOI Slave
    crate::arch::x86_64::ports::outb(0x20, 0x20); // EOI Master
}

// =============================================================================
// HANDLERS ASM (IRQ0 PREEMPTS)
// =============================================================================

core::arch::global_asm!(
    r#"
.global timer_handler
.extern timer_handler_inner

// Macro para salvar scratch registers (caller-saved)
.macro PUSH_SCRATCH_REGS
    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
.endm

// Macro para restaurar scratch registers
.macro POP_SCRATCH_REGS
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax
.endm

timer_handler:
    // 1. Salvar contexto volátil (scratch)
    PUSH_SCRATCH_REGS

    // 2. Chamar handler Rust (manda EOI e talvez schedule)
    call timer_handler_inner

    // 3. Restaurar contexto volátil
    POP_SCRATCH_REGS

    // 4. Retornar da interrupção
    iretq
"#
);

extern "C" {
    fn timer_handler();
}

/// Handler Rust do Timer (chamado pelo ASM)
///
/// Este handler é responsável por:
/// 1. Incrementar contador de ticks do sistema (jiffies).
/// 2. Enviar EOI para o PIC.
///
/// NOTA: Preempção não implementada ainda.
/// Para preempção real, precisamos modificar o assembly do timer handler
/// para salvar contexto completo antes de fazer context switch.
/// Atualmente o sistema usa cooperative multitasking via yield_now().

#[no_mangle]
pub extern "C" fn timer_handler_inner() {
    // 1. Incrementar contador de jiffies (usado para sleep, timeouts, etc)
    crate::core::time::jiffies::inc_jiffies();

    // 2. Enviar EOI para o PIC (Master = 0x20)
    crate::arch::x86_64::ports::outb(0x20, 0x20);

    // TODO: Implementar preempção no futuro
    // Isso requer modificar o assembly do timer handler para:
    // 1. Salvar TODOS os registradores (não apenas scratch)
    // 2. Verificar se veio de userspace
    // 3. Fazer context switch seguro
}

/// Inicializa e remapeia o PIC (Programmable Interrupt Controller) 8259
/// Remapeia IRQs 0-15 para Vetores 32-47 para evitar conflito com Exceções da CPU (0-31).
///
/// # Safety
///
/// Realiza operações de I/O port inseguras.
pub unsafe fn init_pics() {
    use crate::arch::x86_64::ports::outb;

    let pic1_command = 0x20;
    let pic1_data = 0x21;
    let pic2_command = 0xa0;
    let pic2_data = 0xa1;

    // ICW1: Init
    outb(pic1_command, 0x11);
    crate::arch::x86_64::ports::io_wait();
    outb(pic2_command, 0x11);
    crate::arch::x86_64::ports::io_wait();

    // ICW2: Vector Offset
    outb(pic1_data, 0x20); // Master -> 32
    crate::arch::x86_64::ports::io_wait();
    outb(pic2_data, 0x28); // Slave  -> 40
    crate::arch::x86_64::ports::io_wait();

    // ICW3: Cascading
    outb(pic1_data, 4);
    crate::arch::x86_64::ports::io_wait();
    outb(pic2_data, 2);
    crate::arch::x86_64::ports::io_wait();

    // ICW4: 8086 Mode
    outb(pic1_data, 0x01);
    crate::arch::x86_64::ports::io_wait();
    outb(pic2_data, 0x01);
    crate::arch::x86_64::ports::io_wait();

    // OCW1: Mask all interrupts
    outb(pic1_data, 0xff);
    outb(pic2_data, 0xff);

    crate::kinfo!("(Arch) PICs remapped to 32-47 and masked.");
}

/// Habilita uma IRQ específica no PIC (desmascara)
/// IRQ 0-7: PIC1 (master), IRQ 8-15: PIC2 (slave)
pub fn pic_enable_irq(irq: u8) {
    use crate::arch::x86_64::ports::{inb, outb};

    if irq < 8 {
        // Master PIC (porta de dados 0x21)
        let mask = inb(0x21);
        outb(0x21, mask & !(1 << irq));
    } else {
        // Slave PIC (porta de dados 0xA1)
        // Também precisa habilitar IRQ2 no master (cascade)
        let mask_slave = inb(0xA1);
        outb(0xA1, mask_slave & !(1 << (irq - 8)));
        // Garantir que IRQ2 (cascade) está habilitado no master
        let mask_master = inb(0x21);
        outb(0x21, mask_master & !(1 << 2));
    }
}

/// Desabilita uma IRQ específica no PIC (mascara)
pub fn pic_disable_irq(irq: u8) {
    use crate::arch::x86_64::ports::{inb, outb};

    if irq < 8 {
        let mask = inb(0x21);
        outb(0x21, mask | (1 << irq));
    } else {
        let mask = inb(0xA1);
        outb(0xA1, mask | (1 << (irq - 8)));
    }
}

// =============================================================================
// EXCEPTION HANDLERS
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
    crate::kerror!("RIP:", stack_frame.instruction_pointer);
    crate::kerror!("Error Code:", error_code);
    loop {
        crate::arch::Cpu::halt();
    }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: ExceptionStackFrame, error_code: u64) {
    let cr2: u64;
    let cr3: u64;
    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
    }

    crate::kerror!("========================================");
    crate::kerror!("EXCEPTION: PAGE FAULT (#PF)");
    crate::kerror!("========================================");
    crate::kerror!("CR2 (Faulting Addr):", cr2);
    crate::kerror!("CR3 (Page Table):", cr3);
    crate::kerror!("Error Code:", error_code);
    crate::kerror!("RIP:", stack_frame.instruction_pointer);
    crate::kerror!("CS:", stack_frame.code_segment);
    crate::kerror!("RFLAGS:", stack_frame.cpu_flags);
    crate::kerror!("RSP:", stack_frame.stack_pointer);
    crate::kerror!("SS:", stack_frame.stack_segment);

    // Decodificar error code:
    // Bit 0 (P): 0 = página não presente, 1 = proteção violada
    // Bit 1 (W): 0 = leitura, 1 = escrita
    // Bit 2 (U): 0 = kernel, 1 = user mode
    // Bit 3 (R): 1 = reserved bit violation
    // Bit 4 (I): 1 = instruction fetch
    let p = (error_code & 1) != 0;
    let w = (error_code & 2) != 0;
    let u = (error_code & 4) != 0;
    let r = (error_code & 8) != 0;
    let i = (error_code & 16) != 0;

    crate::kerror!("----------------------------------------");
    crate::kerror!("[PF] Error Code Decode:");
    crate::kerror!("[PF] P(present):", if p { 1u64 } else { 0u64 });
    crate::kerror!("[PF] W(write):", if w { 1u64 } else { 0u64 });
    crate::kerror!("[PF] U(user):", if u { 1u64 } else { 0u64 });
    crate::kerror!("[PF] R(rsvd):", if r { 1u64 } else { 0u64 });
    crate::kerror!("[PF] I(instr):", if i { 1u64 } else { 0u64 });

    // Interpretação humana
    if p {
        crate::kerror!("[PF] Causa: Violação de PROTEÇÃO (página presente mas acesso negado)");
    } else {
        crate::kerror!("[PF] Causa: Página NÃO MAPEADA");
    }

    if u {
        crate::kerror!("[PF] Contexto: USER MODE tentou acessar");
    } else {
        crate::kerror!("[PF] Contexto: KERNEL MODE tentou acessar");
    }

    if w {
        crate::kerror!("[PF] Operação: ESCRITA");
    } else if i {
        crate::kerror!("[PF] Operação: INSTRUCTION FETCH");
    } else {
        crate::kerror!("[PF] Operação: LEITURA");
    }

    crate::kerror!("========================================");

    loop {
        crate::arch::Cpu::halt();
    }
}
