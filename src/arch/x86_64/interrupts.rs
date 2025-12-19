//! Interrupt Handlers
//!
//! Handlers de interrupção e exceções para x86_64.

use core::arch::global_asm;

/// Stack frame de interrupção
#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

// Definir handlers em assembly global
global_asm!(
    ".global divide_by_zero_handler",
    "divide_by_zero_handler:",
    "    push 0",
    "    push 0",
    "    jmp common_exception_handler",
    ".global invalid_opcode_handler",
    "invalid_opcode_handler:",
    "    push 0",
    "    push 6",
    "    jmp common_exception_handler",
    ".global general_protection_fault_handler",
    "general_protection_fault_handler:",
    "    push 13",
    "    jmp common_exception_handler",
    ".global page_fault_handler",
    "page_fault_handler:",
    "    push 14",
    "    jmp common_exception_handler",
    ".global common_exception_handler",
    "common_exception_handler:",
    "    push rax",
    "    push rbx",
    "    push rcx",
    "    push rdx",
    "    push rsi",
    "    push rdi",
    "    push rbp",
    "    push r8",
    "    push r9",
    "    push r10",
    "    push r11",
    "    push r12",
    "    push r13",
    "    push r14",
    "    push r15",
    "    mov rdi, rsp",
    "    call exception_handler_rust",
    "    pop r15",
    "    pop r14",
    "    pop r13",
    "    pop r12",
    "    pop r11",
    "    pop r10",
    "    pop r9",
    "    pop r8",
    "    pop rbp",
    "    pop rdi",
    "    pop rsi",
    "    pop rdx",
    "    pop rcx",
    "    pop rbx",
    "    pop rax",
    "    add rsp, 16",
    "    iretq",
    ".global timer_interrupt_handler",
    "timer_interrupt_handler:",
    "    push rax",
    "    push rbx",
    "    push rcx",
    "    push rdx",
    "    push rsi",
    "    push rdi",
    "    push rbp",
    "    push r8",
    "    push r9",
    "    push r10",
    "    push r11",
    "    push r12",
    "    push r13",
    "    push r14",
    "    push r15",
    "    call timer_handler_rust",
    "    pop r15",
    "    pop r14",
    "    pop r13",
    "    pop r12",
    "    pop r11",
    "    pop r10",
    "    pop r9",
    "    pop r8",
    "    pop rbp",
    "    pop rdi",
    "    pop rsi",
    "    pop rdx",
    "    pop rcx",
    "    pop rbx",
    "    pop rax",
    "    iretq",
    ".global keyboard_interrupt_handler",
    "keyboard_interrupt_handler:",
    "    push rax",
    "    push rbx",
    "    push rcx",
    "    push rdx",
    "    push rsi",
    "    push rdi",
    "    push rbp",
    "    push r8",
    "    push r9",
    "    push r10",
    "    push r11",
    "    push r12",
    "    push r13",
    "    push r14",
    "    push r15",
    "    call keyboard_handler_rust",
    "    pop r15",
    "    pop r14",
    "    pop r13",
    "    pop r12",
    "    pop r11",
    "    pop r10",
    "    pop r9",
    "    pop r8",
    "    pop rbp",
    "    pop rdi",
    "    pop rsi",
    "    pop rdx",
    "    pop rcx",
    "    pop rbx",
    "    pop rax",
    "    iretq",
    ".global syscall_interrupt_handler",
    "syscall_interrupt_handler:",
    "    push rbp",
    "    push rbx",
    "    push r12",
    "    push r13",
    "    push r14",
    "    push r15",
    "    mov rdi, rax", // num
    "    mov rsi, rdi", // arg1
    "    mov rdx, rsi", // arg2
    "    mov rcx, rdx", // arg3
    "    call syscall_handler_rust",
    "    pop r15",
    "    pop r14",
    "    pop r13",
    "    pop r12",
    "    pop rbx",
    "    pop rbp",
    "    iretq",
);

// Declarar símbolos externos
unsafe extern "C" {
    pub fn divide_by_zero_handler();
    pub fn invalid_opcode_handler();
    pub fn general_protection_fault_handler();
    pub fn page_fault_handler();
    pub fn timer_interrupt_handler();
    pub fn keyboard_interrupt_handler();
    pub fn syscall_interrupt_handler();
}

/// Handler Rust para exceções
#[unsafe(no_mangle)]
extern "C" fn exception_handler_rust(stack_ptr: u64) {
    use crate::drivers::legacy::serial;

    // Stack frame contém: r15, r14, ..., rax, error_code, exception_num
    let exception_num = unsafe { *((stack_ptr + 15 * 8) as *const u64) };
    let error_code = unsafe { *((stack_ptr + 15 * 8 + 8) as *const u64) };

    serial::println("\n=== EXCEPTION ===");

    match exception_num {
        0 => serial::println("Tipo: Division by Zero (#DE)"),
        6 => serial::println("Tipo: Invalid Opcode (#UD)"),
        13 => {
            serial::println("Tipo: General Protection Fault (#GP)");
            serial::print("Error Code: 0x");
            print_hex(error_code as usize);
            serial::println("");
        }
        14 => {
            serial::println("Tipo: Page Fault (#PF)");

            // Ler CR2 (endereço que causou o fault)
            let cr2: u64;
            unsafe {
                core::arch::asm!("mov {}, cr2", out(reg) cr2);
            }

            serial::print("Endereco: 0x");
            print_hex(cr2 as usize);
            serial::println("");

            serial::print("Erro: ");
            if error_code & 0x1 != 0 {
                serial::print("Protecao ");
            } else {
                serial::print("Nao-presente ");
            }
            if error_code & 0x2 != 0 {
                serial::print("Escrita ");
            } else {
                serial::print("Leitura ");
            }
            if error_code & 0x4 != 0 {
                serial::println("Usuario");
            } else {
                serial::println("Kernel");
            }
        }
        _ => {
            serial::print("Tipo: Desconhecida (");
            print_hex(exception_num as usize);
            serial::println(")");
        }
    }

    serial::println("=================\n");

    // Halt
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Imprime número em hexadecimal
fn print_hex(mut n: usize) {
    use crate::drivers::legacy::serial;

    if n == 0 {
        serial::print("0");
        return;
    }

    let mut buf = [0u8; 16];
    let mut i = 0;
    while n > 0 {
        let d = n % 16;
        buf[i] = if d < 10 {
            b'0' + d as u8
        } else {
            b'a' + (d - 10) as u8
        };
        n /= 16;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        serial::write_byte(buf[i]);
    }
}

/// Handler Rust para timer
#[unsafe(no_mangle)]
extern "C" fn timer_handler_rust() {
    use crate::core::process::{ProcessState, PROCESS_MANAGER};
    use crate::core::scheduler::SCHEDULER;

    // Incrementar contador do timer
    crate::drivers::timer::pit::timer_handler();

    // Verificar se quantum expirou
    let should_switch = SCHEDULER.lock().tick();

    if should_switch {
        let mut pm = PROCESS_MANAGER.lock();

        // Se há processo atual
        if let Some(current_pid) = pm.current_pid {
            // Marcar atual como Ready
            if let Some(current) = pm.processes.iter_mut().find(|p| p.pid == current_pid) {
                if current.state == ProcessState::Running {
                    current.state = ProcessState::Ready;
                }
            }

            // Pegar próximo processo Ready
            if let Some(next) = pm
                .processes
                .iter_mut()
                .find(|p| p.state == ProcessState::Ready)
            {
                let next_pid = next.pid;
                next.state = ProcessState::Running;

                // Atualizar current_pid
                let old_pid = pm.current_pid;
                pm.current_pid = Some(next_pid);

                // Context switch com unsafe
                // SAFETY: Ambos os PIDs são válidos e processos não são removidos durante switch
                unsafe {
                    pm.switch_context(current_pid, next_pid);
                }
            }
        } else {
            // Nenhum processo atual, pegar primeiro Ready
            if let Some(first) = pm
                .processes
                .iter_mut()
                .find(|p| p.state == ProcessState::Ready)
            {
                first.state = ProcessState::Running;
                pm.current_pid = Some(first.pid);
            }
        }
    }
}

/// Handler Rust para keyboard
#[unsafe(no_mangle)]
extern "C" fn keyboard_handler_rust() {
    use crate::drivers::{input_buffer, keyboard, legacy::serial};

    // Debug mínimo
    serial::print("K");

    // Ler scancode
    let scancode = keyboard::read_scancode();

    // Processar scancode
    if let Some(ch) = keyboard::process_scancode(scancode) {
        // Adicionar ao buffer
        input_buffer::INPUT_BUFFER.lock().push(ch);

        // Echo no serial
        if ch == '\n' {
            serial::println("");
        } else if ch == '\x08' {
            // Backspace
            serial::print("\x08 \x08");
        } else {
            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            serial::print(s);
        }
    }

    // Enviar EOI ao PIC
    crate::drivers::pic::send_eoi(1);
}

/// Formata byte como hex
fn format_hex(value: u8, buf: &mut [u8; 16]) -> &str {
    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";
    buf[0] = HEX_CHARS[(value >> 4) as usize];
    buf[1] = HEX_CHARS[(value & 0x0F) as usize];
    buf[2] = 0;
    unsafe { core::str::from_utf8_unchecked(&buf[0..2]) }
}

/// Handler Rust para syscalls (int 0x80)
#[unsafe(no_mangle)]
extern "C" fn syscall_handler_rust(num: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    crate::syscall::syscall_handler(num, arg1, arg2, arg3)
}
