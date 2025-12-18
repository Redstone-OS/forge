//! Context Switching
//!
//! Assembly code para trocar contexto entre processos.

use super::ProcessContext;
use core::arch::global_asm;

// Assembly para context switch
global_asm!(
    ".global switch_context",
    "switch_context:",
    // RDI = &mut old_context (salvar aqui)
    // RSI = &new_context (carregar daqui)

    // Salvar contexto atual em [RDI]
    "mov [rdi + 0x00], rax",
    "mov [rdi + 0x08], rbx",
    "mov [rdi + 0x10], rcx",
    "mov [rdi + 0x18], rdx",
    "mov [rdi + 0x20], rsi",
    "mov [rdi + 0x28], rdi",
    "mov [rdi + 0x30], rbp",
    "mov [rdi + 0x38], rsp",
    "mov [rdi + 0x40], r8",
    "mov [rdi + 0x48], r9",
    "mov [rdi + 0x50], r10",
    "mov [rdi + 0x58], r11",
    "mov [rdi + 0x60], r12",
    "mov [rdi + 0x68], r13",
    "mov [rdi + 0x70], r14",
    "mov [rdi + 0x78], r15",
    // Salvar RIP (endereço de retorno)
    "mov rax, [rsp]",
    "mov [rdi + 0x80], rax",
    // Salvar RFLAGS
    "pushfq",
    "pop rax",
    "mov [rdi + 0x88], rax",
    // Carregar novo contexto de [RSI]
    "mov rax, [rsi + 0x00]",
    "mov rbx, [rsi + 0x08]",
    "mov rcx, [rsi + 0x10]",
    "mov rdx, [rsi + 0x18]",
    // RSI e RDI depois
    "mov rbp, [rsi + 0x30]",
    "mov rsp, [rsi + 0x38]",
    "mov r8,  [rsi + 0x40]",
    "mov r9,  [rsi + 0x48]",
    "mov r10, [rsi + 0x50]",
    "mov r11, [rsi + 0x58]",
    "mov r12, [rsi + 0x60]",
    "mov r13, [rsi + 0x68]",
    "mov r14, [rsi + 0x70]",
    "mov r15, [rsi + 0x78]",
    // Carregar RFLAGS
    "mov r8, [rsi + 0x88]",
    "push r8",
    "popfq",
    // Carregar RIP (empurrar na stack para ret)
    "mov r8, [rsi + 0x80]",
    "push r8",
    // Carregar RSI e RDI por último
    "mov rdi, [rsi + 0x28]",
    "mov rsi, [rsi + 0x20]",
    // Retornar para novo RIP
    "ret",
);

unsafe extern "C" {
    /// Troca contexto de old para new
    pub fn switch_context(old: &mut ProcessContext, new: &ProcessContext);
}
