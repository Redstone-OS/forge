//! # Subsistema de Interrupções e Exceções (x86_64)
//!
//! Este módulo gerencia a comunicação entre o hardware e o kernel através da
//! Tabela de Descritores de Interrupções (IDT). Ele é responsável por capturar
//! eventos críticos da CPU, gerenciar interrupções de dispositivos (IRQs) e
//! servir como o ponto de entrada principal para o agendamento preemptivo.
//!
//! ## Arquitetura:
//! - **Exceções (0-31):** Tratamento de erros síncronos (Page Faults, GPF, etc).
//! - **IRQs (32-47):** Interrupções externas remapeadas via PIC/APIC.
//! - **Preempção:** O Timer (IRQ 0) é o gatilho que permite ao kernel retomar o
//!   controle da CPU em intervalos regulares.
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

// Inclui o arquivo assembly externo contendo os trampolins e handlers de baixo nível
core::arch::global_asm!(include_str!("interrupts.s"));

// Declaração dos wrappers definidos no assembly
extern "C" {
    fn divide_error_wrapper();
    fn invalid_opcode_wrapper();
    fn page_fault_wrapper();
    fn general_protection_wrapper();
    fn double_fault_wrapper();
    fn breakpoint_wrapper();
    fn timer_handler(); // Definido em interrupts.s
}

// (Helpers removidos pois já existem em sched/core/cpu.rs e scheduler.rs)

/// Inicializa a Tabela de Descritores de Interrupção (IDT).
/// Deve ser chamado no boot antes de habilitar interrupções.
pub fn init_idt() {
    let idt = unsafe { &mut *core::ptr::addr_of_mut!(IDT) };

    // Debug: Print handler address
    crate::kinfo!(
        "(IDT) Divide Error Wrapper Addr:",
        divide_error_wrapper as *const () as u64
    );

    idt.set_handler(0, divide_error_wrapper as *const () as u64);
    idt.set_handler(3, breakpoint_wrapper as *const () as u64);
    idt.set_handler(6, invalid_opcode_wrapper as *const () as u64);
    // Double Fault usa IST 1 para garantir stack segura
    idt.set_handler_ist(8, double_fault_wrapper as *const () as u64, 1);
    idt.set_handler(13, general_protection_wrapper as *const () as u64);
    idt.set_handler(14, page_fault_wrapper as *const () as u64);

    // Remapear IRQs (PIC) -> 32..47
    // Timer (IRQ 0) -> 32
    // Agora usamos o handler asm 'timer_handler' para permitir preempção
    idt.set_handler(32, timer_handler as *const () as u64);
    idt.set_handler(33, keyboard_interrupt_handler as *const () as u64);
    idt.set_handler(44, mouse_interrupt_handler as *const () as u64);

    unsafe {
        idt.load();
    }
}

// =============================================================================
// HANDLERS ASM (IRQs Simples)
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
// HANDLERS RUST (INNER)
// =============================================================================

/// Handler Rust do Timer (chamado pelo ASM)
///
/// Este handler é responsável por:
/// 1. Incrementar contador de ticks do sistema (jiffies).
/// 2. Enviar EOI para o PIC.
#[no_mangle]
pub extern "C" fn timer_handler_inner() {
    // 1. Incrementar contador de jiffies (usado para sleep, timeouts, etc)
    crate::core::time::jiffies::inc_jiffies();

    // 2. Notificar o scheduler sobre a passagem de tempo (Time-Slicing)
    crate::sched::core::scheduler::timer_tick();

    // 3. Verificar se há tasks para acordar na SleepQueue
    crate::sched::core::sleep_queue::check_sleep_queue();

    // 4. Enviar EOI para o PIC (Master = 0x20)
    crate::arch::x86_64::ports::outb(0x20, 0x20);
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
// HANDLERS DE EXCEÇÕES (CORE)
// =============================================================================

#[no_mangle]
pub extern "C" fn divide_error_handler_inner(stack_frame: *const ExceptionStackFrame) {
    // Reconstruímos a referência a partir do ponteiro
    let frame = unsafe { &*stack_frame };
    handle_fault("Divide Error (#DE)", frame, None, None);
}

#[no_mangle]
pub extern "C" fn invalid_opcode_handler_inner(stack_frame: *const ExceptionStackFrame) {
    let frame = unsafe { &*stack_frame };
    handle_fault("Invalid Opcode (#UD)", frame, None, None);
}

#[no_mangle]
pub extern "C" fn breakpoint_handler_inner(stack_frame: *const ExceptionStackFrame) {
    let frame = unsafe { &*stack_frame };
    crate::kinfo!("EXCEPTION: BREAKPOINT at", frame.instruction_pointer);
}

#[no_mangle]
pub extern "C" fn page_fault_handler_inner(
    stack_frame: *const ExceptionStackFrame,
    error_code: u64,
) {
    let frame = unsafe { &*stack_frame };
    let cr2: u64;
    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
    }
    handle_fault("Page Fault (#PF)", frame, Some(error_code), Some(cr2));
}

#[no_mangle]
pub extern "C" fn general_protection_handler_inner(
    stack_frame: *const ExceptionStackFrame,
    error_code: u64,
) {
    let frame = unsafe { &*stack_frame };
    handle_fault(
        "General Protection Fault (#GP)",
        frame,
        Some(error_code),
        None,
    );
}

#[no_mangle]
pub extern "C" fn double_fault_handler_inner(
    stack_frame: *const ExceptionStackFrame,
    _error_code: u64,
) -> ! {
    let frame = unsafe { &*stack_frame };
    crate::kerror!("EXCEPTION: DOUBLE FAULT (#DF)");
    crate::kerror!("RIP:", frame.instruction_pointer);
    panic!("DOUBLE FAULT - Stack Overflow ou corrupção crítica detectada.");
}

/// Trata falhas de CPU decidindo se deve matar o processo ou dar Panic no kernel.
#[allow(dead_code)]
fn handle_fault(
    name: &str,
    stack_frame: &ExceptionStackFrame,
    error_code: Option<u64>,
    extra_info: Option<u64>,
) {
    let is_user = (stack_frame.code_segment & 3) == 3;

    if is_user {
        crate::kerror!("!!! FALHA EM PROCESSO DE USUÁRIO !!!");
        crate::kerror!("Exceção:", name);
        crate::kerror!("RIP:", stack_frame.instruction_pointer);
        crate::kerror!("CS:", stack_frame.code_segment);
        if let Some(err) = error_code {
            crate::kerror!("Error Code:", err);
        }
        if let Some(info) = extra_info {
            crate::kerror!("Extra Info (ex: CR2):", info);
        }
        crate::kerror!("Ação: Encerrando processo infrator.");

        // Encerra a task atual e pula para a próxima via scheduler
        crate::sched::core::scheduler::exit_current(-1);
    } else {
        crate::kerror!("!!! KERNEL PANIC !!!");
        crate::kerror!("Exceção crítica no modo kernel:", name);
        crate::kerror!("RIP:", stack_frame.instruction_pointer);
        if let Some(err) = error_code {
            crate::kerror!("Error Code:", err);
        }
        panic!("Falha Crítica no Kernel!");
    }
}
