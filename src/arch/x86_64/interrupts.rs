//! Stubs de Interrupção em Assembly (Naked Functions).
//!
//! Rust não consegue salvar todos os registradores automaticamente na entrada
//! de uma interrupção. Precisamos de assembly puro para criar o `InterruptStackFrame`
//! e chamar o handler Rust seguro.

use core::arch::naked_asm;

// Macro para criar stubs de exceção sem código de erro
macro_rules! handler_no_err {
    ($name:ident, $handler_fn:ident) => {
        #[naked]
        pub extern "C" fn $name() {
            unsafe {
                naked_asm!(
                    "push 0", // Fake error code para alinhar stack
                    "push rbp",
                    "push r15", "push r14", "push r13", "push r12", "push r11", "push r10", "push r9", "push r8",
                    "push rdi", "push rsi", "push rdx", "push rcx", "push rbx", "push rax",
                    "mov rdi, rsp", // Passar ponteiro da stack como argumento para o Rust
                    "call {}",      // Chamar handler Rust
                    "pop rax", "pop rbx", "pop rcx", "pop rdx", "pop rsi", "pop rdi",
                    "pop r8", "pop r9", "pop r10", "pop r11", "pop r12", "pop r13", "pop r14", "pop r15",
                    "pop rbp",
                    "add rsp, 8", // Remover error code
                    "iretq",
                    sym $handler_fn
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
                naked_asm!(
                    // Error code já está na stack
                    "push rbp",
                    "push r15", "push r14", "push r13", "push r12", "push r11", "push r10", "push r9", "push r8",
                    "push rdi", "push rsi", "push rdx", "push rcx", "push rbx", "push rax",
                    "mov rdi, rsp",
                    "call {}",
                    "pop rax", "pop rbx", "pop rcx", "pop rdx", "pop rsi", "pop rdi",
                    "pop r8", "pop r9", "pop r10", "pop r11", "pop r12", "pop r13", "pop r14", "pop r15",
                    "pop rbp",
                    "add rsp, 8", // Remover error code
                    "iretq",
                    sym $handler_fn
                );
            }
        }
    };
}

// Handlers Rust (Seguros)
use crate::arch::x86_64::idt::ContextFrame;

extern "C" fn breakpoint_handler_impl(frame: &ContextFrame) {
    crate::kinfo!("EXCEPTION: BREAKPOINT at {:#x}", frame.rip);
}

extern "C" fn double_fault_handler_impl(frame: &ContextFrame) {
    crate::kerror!("EXCEPTION: DOUBLE FAULT at {:#x}\n{:#?}", frame.rip, frame);
    crate::arch::platform::cpu::X64Cpu::hang();
}

extern "C" fn page_fault_handler_impl(frame: &ContextFrame) {
    let cr2: u64;
    unsafe { core::arch::asm!("mov {}, cr2", out(reg) cr2) };
    crate::kerror!(
        "EXCEPTION: PAGE FAULT at {:#x} accessing {:#x}",
        frame.rip,
        cr2
    );
    crate::arch::platform::cpu::X64Cpu::hang();
}

extern "C" fn general_protection_fault_handler_impl(frame: &ContextFrame) {
    crate::kerror!(
        "EXCEPTION: GPF at {:#x} Code: {:#x}",
        frame.rip,
        frame.error_code
    );
    crate::arch::platform::cpu::X64Cpu::hang();
}

// Stubs exportados
handler_no_err!(breakpoint_handler, breakpoint_handler_impl);
handler_with_err!(double_fault_handler, double_fault_handler_impl);
handler_with_err!(page_fault_handler, page_fault_handler_impl);
handler_with_err!(
    general_protection_fault_handler,
    general_protection_fault_handler_impl
);
