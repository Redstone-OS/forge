//! Stubs de Interrupção em Assembly.
//!
//! Implementação usando `asm!` dentro de funções `#[naked]`, conforme
//! o padrão atual do Rust Nightly (substituindo o obsoleto `naked_asm!`).

use crate::arch::traits::CpuOps;
use crate::arch::x86_64::idt::ContextFrame;
use core::arch::asm; // Necessário para .hang()

// Macro para criar stubs de exceção SEM código de erro (push 0 manual)
macro_rules! handler_no_err {
    ($name:ident, $handler_fn:ident) => {
        #[naked]
        pub extern "C" fn $name() {
            unsafe {
                asm!(
                    "push 0",       // Fake error code para alinhar stack
                    "push rbp",     // Salvar registradores gerais
                    "push r15", "push r14", "push r13", "push r12",
                    "push r11", "push r10", "push r9",  "push r8",
                    "push rdi", "push rsi", "push rdx", "push rcx", "push rbx", "push rax",

                    "mov rdi, rsp", // Arg 1 (frame): Ponteiro para a stack atual
                    "call {handler}", // Chamar handler Rust seguro

                    "pop rax", "pop rbx", "pop rcx", "pop rdx", "pop rsi", "pop rdi",
                    "pop r8",  "pop r9",  "pop r10", "pop r11",
                    "pop r12", "pop r13", "pop r14", "pop r15",
                    "pop rbp",
                    "add rsp, 8",   // Remover fake error code
                    "iretq",        // Retorno de interrupção
                    handler = sym $handler_fn,
                    options(noreturn)
                );
            }
        }
    };
}

// Macro para exceções que JÁ empilham erro (ex: Page Fault)
macro_rules! handler_with_err {
    ($name:ident, $handler_fn:ident) => {
        #[naked]
        pub extern "C" fn $name() {
            unsafe {
                asm!(
                    // Error code já está na stack (empilhado pela CPU)
                    "push rbp",
                    "push r15", "push r14", "push r13", "push r12",
                    "push r11", "push r10", "push r9",  "push r8",
                    "push rdi", "push rsi", "push rdx", "push rcx", "push rbx", "push rax",

                    "mov rdi, rsp",
                    "call {handler}",

                    "pop rax", "pop rbx", "pop rcx", "pop rdx", "pop rsi", "pop rdi",
                    "pop r8",  "pop r9",  "pop r10", "pop r11",
                    "pop r12", "pop r13", "pop r14", "pop r15",
                    "pop rbp",
                    "add rsp, 8",   // Remover error code real
                    "iretq",
                    handler = sym $handler_fn,
                    options(noreturn)
                );
            }
        }
    };
}

// --- Implementações Rust Seguras ---

extern "C" fn breakpoint_handler_impl(frame: &ContextFrame) {
    crate::kinfo!("EXCEPTION: BREAKPOINT at {:#x}", frame.rip);
}

extern "C" fn double_fault_handler_impl(frame: &ContextFrame) {
    crate::kerror!("EXCEPTION: DOUBLE FAULT\n{:#?}", frame);
    // Trava o sistema usando a abstração de CPU
    crate::arch::platform::Cpu::hang();
}

extern "C" fn page_fault_handler_impl(frame: &ContextFrame) {
    let cr2: u64;
    // Ler endereço que causou a falha (CR2)
    unsafe {
        asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
    };

    crate::kerror!(
        "EXCEPTION: PAGE FAULT at {:#x} accessing {:#x}\nError Code: {:?}",
        frame.rip,
        cr2,
        frame.error_code
    );
    crate::arch::platform::Cpu::hang();
}

extern "C" fn general_protection_fault_handler_impl(frame: &ContextFrame) {
    crate::kerror!(
        "EXCEPTION: GPF at {:#x} Code: {:#x}",
        frame.rip,
        frame.error_code
    );
    crate::arch::platform::Cpu::hang();
}

extern "C" fn timer_handler_impl(_frame: &ContextFrame) {
    // Chama o driver do timer para processar ticks e scheduling
    crate::drivers::timer::handle_timer_interrupt();
}

// --- Geração dos Stubs ---

handler_no_err!(breakpoint_handler, breakpoint_handler_impl);
handler_with_err!(double_fault_handler, double_fault_handler_impl);
handler_with_err!(page_fault_handler, page_fault_handler_impl);
handler_with_err!(
    general_protection_fault_handler,
    general_protection_fault_handler_impl
);
handler_no_err!(timer_handler, timer_handler_impl);
