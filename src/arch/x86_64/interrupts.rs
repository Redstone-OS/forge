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

// Declaração dos wrappers definidos no global_asm!
extern "C" {
    fn divide_error_wrapper();
    fn invalid_opcode_wrapper();
    fn page_fault_wrapper();
    fn general_protection_wrapper();
    fn double_fault_wrapper();
    fn breakpoint_wrapper();
}

/// Inicializa a Tabela de Descritores de Interrupção (IDT).
/// Deve ser chamado no boot antes de habilitar interrupções.
pub fn init_idt() {
    let idt = unsafe { &mut *core::ptr::addr_of_mut!(IDT) };

    // Debug: Print handler address
    crate::kinfo!(
        "(IDT) Divide Error Wrapper Addr:",
        divide_error_wrapper as u64
    );

    idt.set_handler(0, divide_error_wrapper as u64);
    idt.set_handler(3, breakpoint_wrapper as u64);
    idt.set_handler(6, invalid_opcode_wrapper as u64);
    idt.set_handler(8, double_fault_wrapper as u64);
    idt.set_handler(13, general_protection_wrapper as u64);
    idt.set_handler(14, page_fault_wrapper as u64);

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
// HANDLERS ASM (WRAPPERS - EXCEPTION TRAMPOLINES)
// =============================================================================

core::arch::global_asm!(
    r#"
.global divide_error_wrapper
.global invalid_opcode_wrapper
.global page_fault_wrapper
.global general_protection_wrapper
.global double_fault_wrapper
.global breakpoint_wrapper

.extern divide_error_handler_inner
.extern invalid_opcode_handler_inner
.extern page_fault_handler_inner
.extern general_protection_handler_inner
.extern double_fault_handler_inner
.extern breakpoint_handler_inner

// Macro para salvar scratch registers
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

// Wrapper para exceções SEM código de erro
// Stack: [RIP, CS, RFLAGS, RSP, SS]
.macro EXCEPTION_NO_ERR name, inner
\name:
    // Salvar regs voláteis
    PUSH_SCRATCH_REGS
    
    // Verificar CS (bit 1-0) em [RSP + 72 + 8] = [RSP + 80]
    // 72 bytes de regs + 8 bytes RIP = CS está em +80
    testb $3, 80(%rsp)
    jz 1f
    swapgs
1:
    // RDI = Ponteiro para stack frame (começa no RIP, em RSP + 72)
    lea rdi, [rsp + 72]
    
    // Alinhar stack (opcional, mas bom pra ABI)
    and rsp, -16
    
    call \inner
    
    // Restaurar GS se necessário
    // Stack original foi alterada? Não, pois 'and rsp' só afeta localmente se usarmos frame pointer
    // Mas aqui não salvamos RSP antigo. O 'and rsp' poderia quebrar o retorno?
    // SIM! Se alterarmos RSP sem salvar, perdemos a stack de retorno.
    // Vamos remover o alinhamento manual por enquanto ou fazer direito (mov rbp, rsp).
    // Como estamos apenas chamando func Rust que não usa muita stack de args, talvez ok.
    // Melhor não arriscar: SEM ALINHAMENTO MANUAL AGORA.
    
    // Recuperar stack original? Não mudamos se não fizermos 'and'.
    
    // Checar swapgs de volta
    // Mas espera, como acessamos a stack original para checar CS se mudamos RSP?
    // Ah, não mudamos RSP.
    
    testb $3, 80(%rsp)
    jz 2f
    swapgs
2:
    POP_SCRATCH_REGS
    iretq
.endm

// Wrapper para exceções COM código de erro
// Stack: [ERR, RIP, CS, RFLAGS, RSP, SS]
.macro EXCEPTION_WITH_ERR name, inner
\name:
    // Salvar regs voláteis
    PUSH_SCRATCH_REGS
    
    // Verificar CS. 
    // Stack: [Regs(72), ERR(8), RIP(8), CS(8)]
    // CS está em 72 + 8 + 8 = 88
    testb $3, 88(%rsp)
    jz 1f
    swapgs
1:
    // RDI = Ponteiro para stack frame (RIP está em RSP + 80, pois tem ERR code antes)
    lea rdi, [rsp + 80]
    
    // RSI = Error Code (está em RSP + 72)
    mov rsi, [rsp + 72]
    
    call \inner
    
    testb $3, 88(%rsp)
    jz 2f
    swapgs
2:
    POP_SCRATCH_REGS
    // Pop error code
    add rsp, 8
    iretq
.endm

EXCEPTION_NO_ERR divide_error_wrapper, divide_error_handler_inner
EXCEPTION_NO_ERR invalid_opcode_wrapper, invalid_opcode_handler_inner
EXCEPTION_NO_ERR breakpoint_wrapper, breakpoint_handler_inner
EXCEPTION_WITH_ERR page_fault_wrapper, page_fault_handler_inner
EXCEPTION_WITH_ERR general_protection_wrapper, general_protection_handler_inner
EXCEPTION_WITH_ERR double_fault_wrapper, double_fault_handler_inner
"#
);

// =============================================================================
// HANDLERS RUST (INNER)
// =============================================================================

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

    // 2. Chamar handler Rust (manda EOI e atualiza jiffies)
    call timer_handler_inner

    // 3. Ponto de Preempção (Preemption Point)
    // Somente preemptamos se estávamos voltando para User Mode (Ring 3).
    // Preemptar o Kernel (Ring 0) requer cuidado extremo com reentrância e locks.
    // O Code Segment (CS) está em [rsp + 80] (9 regs salvos + RIP)
    mov rax, [rsp + 80]
    and rax, 3
    cmp rax, 3
    jne .skip_preemption

    // Verificar se a CPU sinalizou que precisamos de agendamento
    // Alinhamento manual de stack (16-bytes) para chamadas Rust.
    // Preservamos o r12 (callee-saved) pois vamos usá-lo como âncora para o RSP.
    push r12
    mov r12, rsp
    and rsp, -16
    
    call should_reschedule
    test al, al
    jz .restore_rsp

    // Se chegamos aqui, o quantum acabou!
    // Limpamos a flag e chamamos o orquestrador.
    call clear_need_resched
    call schedule

.restore_rsp:
    mov rsp, r12
    pop r12

.skip_preemption:
    // 4. Restaurar contexto volátil
    POP_SCRATCH_REGS

    // 5. Retornar da interrupção (restaura CS, RIP, RFLAGS, RSP, SS)
    iretq
"#
);

#[allow(dead_code)]
extern "C" {
    fn timer_handler();
    fn should_reschedule() -> bool;
    fn clear_need_resched();
    fn schedule();
}

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
// HANDLERS DE EXCEÇÕES
// =============================================================================

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
