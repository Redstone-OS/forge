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

// --- Implementações Rust Seguras (logs em PT-BR) ---
// IMPORTANTE: Handlers críticos (#GPF, #UD, #DF) NÃO devem usar kerror!/formatação
// porque podem causar outra exceção se a heap/SSE não estiver pronta.

extern "C" fn breakpoint_handler_impl(frame: &ContextFrame) {
    crate::kwarn!("(Int) EXCEÇÃO: BREAKPOINT em RIP={:#x}", frame.rip);
}

/// Handler de Double Fault - SEGURO (sem formatação)
extern "C" fn double_fault_handler_impl(frame: &ContextFrame) {
    // Usar escrita raw para evitar #UD em cascata
    crate::drivers::serial::write_str_raw("\r\n[FATAL] DOUBLE FAULT at RIP=");
    crate::drivers::serial::write_hex_raw(frame.rip);
    crate::drivers::serial::write_str_raw(" RSP=");
    crate::drivers::serial::write_hex_raw(frame.rsp);
    crate::drivers::serial::write_newline_raw();

    // Halt simples sem SSE
    loop {
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
    }
}

extern "C" fn page_fault_handler_impl(frame: &ContextFrame) {
    let cr2: u64;
    unsafe {
        asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
    };

    // Usar escrita raw para evitar #UD
    crate::drivers::serial::write_str_raw("\r\n[FATAL] PAGE FAULT at RIP=");
    crate::drivers::serial::write_hex_raw(frame.rip);
    crate::drivers::serial::write_str_raw(" CR2=");
    crate::drivers::serial::write_hex_raw(cr2);
    crate::drivers::serial::write_str_raw(" ERR=");
    crate::drivers::serial::write_hex_raw(frame.error_code);
    crate::drivers::serial::write_newline_raw();

    // Halt simples
    loop {
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
    }
}

/// Handler de GPF - SEGURO (sem formatação para evitar cascata)
extern "C" fn general_protection_fault_handler_impl(frame: &ContextFrame) {
    // Usar escrita raw - CRÍTICO: não usar kerror! aqui
    crate::drivers::serial::write_str_raw("\r\n[FATAL] GPF at RIP=");
    crate::drivers::serial::write_hex_raw(frame.rip);
    crate::drivers::serial::write_str_raw(" ERR=");
    crate::drivers::serial::write_hex_raw(frame.error_code);
    crate::drivers::serial::write_newline_raw();

    // Halt simples sem usar Cpu::hang() que pode ter código problemático
    loop {
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
    }
}

extern "C" fn timer_handler_impl(_frame: &ContextFrame) {
    // Chama o driver do timer para processar ticks e scheduling
    crate::drivers::timer::handle_timer_interrupt();
}

/// Handler de Invalid Opcode (#UD) - SEGURO (sem formatação)
extern "C" fn invalid_opcode_handler_impl(frame: &ContextFrame) {
    // CRÍTICO: NÃO usar kerror! aqui - pode causar loop infinito de #UD
    crate::drivers::serial::write_str_raw("\r\n[FATAL] INVALID OPCODE at RIP=");
    crate::drivers::serial::write_hex_raw(frame.rip);
    crate::drivers::serial::write_newline_raw();

    // Halt simples
    loop {
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
    }
}

// --- Geração dos Stubs ---

handler_no_err!(breakpoint_handler, breakpoint_handler_impl);
handler_no_err!(invalid_opcode_handler, invalid_opcode_handler_impl); // ADDED
handler_with_err!(double_fault_handler, double_fault_handler_impl);
handler_with_err!(page_fault_handler, page_fault_handler_impl);
handler_with_err!(
    general_protection_fault_handler,
    general_protection_fault_handler_impl
);
handler_no_err!(timer_handler, timer_handler_impl);
