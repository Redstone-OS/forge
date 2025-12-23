//! Stubs de Interrupção em Assembly.

use crate::arch::traits::CpuOps;
use crate::arch::x86_64::idt::ContextFrame;
use core::arch::naked_asm; // CORREÇÃO: Import necessário // CORREÇÃO: Trait para .hang()

// Macros de handler (mantidas idênticas para economizar espaço visual, mas funcionais)
macro_rules! handler_no_err {
    ($name:ident, $handler_fn:ident) => {
        #[naked]
        pub extern "C" fn $name() {
            unsafe {
                naked_asm!(
                    "push 0", "push rbp",
                    "push r15", "push r14", "push r13", "push r12", "push r11", "push r10", "push r9", "push r8",
                    "push rdi", "push rsi", "push rdx", "push rcx", "push rbx", "push rax",
                    "mov rdi, rsp",
                    "call {}",
                    "pop rax", "pop rbx", "pop rcx", "pop rdx", "pop rsi", "pop rdi",
                    "pop r8", "pop r9", "pop r10", "pop r11", "pop r12", "pop r13", "pop r14", "pop r15",
                    "pop rbp", "add rsp, 8", "iretq",
                    sym $handler_fn
                );
            }
        }
    };
}

macro_rules! handler_with_err {
    ($name:ident, $handler_fn:ident) => {
        #[naked]
        pub extern "C" fn $name() {
            unsafe {
                naked_asm!(
                    "push rbp",
                    "push r15", "push r14", "push r13", "push r12", "push r11", "push r10", "push r9", "push r8",
                    "push rdi", "push rsi", "push rdx", "push rcx", "push rbx", "push rax",
                    "mov rdi, rsp",
                    "call {}",
                    "pop rax", "pop rbx", "pop rcx", "pop rdx", "pop rsi", "pop rdi",
                    "pop r8", "pop r9", "pop r10", "pop r11", "pop r12", "pop r13", "pop r14", "pop r15",
                    "pop rbp", "add rsp, 8", "iretq",
                    sym $handler_fn
                );
            }
        }
    };
}

extern "C" fn breakpoint_handler_impl(frame: &ContextFrame) {
    crate::kinfo!("EXCEPTION: BREAKPOINT at {:#x}", frame.rip);
}

extern "C" fn double_fault_handler_impl(frame: &ContextFrame) {
    crate::kerror!("EXCEPTION: DOUBLE FAULT\n{:#?}", frame);
    // CORREÇÃO: Usar o wrapper da plataforma ou trait
    crate::arch::platform::Cpu::hang();
}

extern "C" fn page_fault_handler_impl(frame: &ContextFrame) {
    let cr2: u64;
    unsafe { core::arch::asm!("mov {}, cr2", out(reg) cr2) };
    crate::kerror!(
        "EXCEPTION: PAGE FAULT at {:#x} accessing {:#x}",
        frame.rip,
        cr2
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
    crate::drivers::timer::handle_timer_interrupt();
}

handler_no_err!(breakpoint_handler, breakpoint_handler_impl);
handler_with_err!(double_fault_handler, double_fault_handler_impl);
handler_with_err!(page_fault_handler, page_fault_handler_impl);
handler_with_err!(
    general_protection_fault_handler,
    general_protection_fault_handler_impl
);
handler_no_err!(timer_handler, timer_handler_impl);
