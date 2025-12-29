//! Context switch
//!
//! Gerencia a troca de contexto entre tasks.

use crate::mm::VirtAddr;

/// Contexto de CPU (registradores salvos)
///
/// NOTA: FpuState temporariamente removido para debug de SSE
/// Com SSE desabilitado no kernel, não precisamos salvar/restaurar estado FPU
#[repr(C)]
pub struct CpuContext {
    // Callee-saved registers (SysV ABI)
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // Stack pointer
    pub rsp: u64,

    // Instruction pointer (return address)
    pub rip: u64,
}

impl CpuContext {
    /// Cria CpuContext zerado - usa const fn para evitar código SSE
    pub const fn new() -> Self {
        Self {
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rsp: 0,
            rip: 0,
        }
    }

    /// Configura para iniciar em função específica
    pub fn setup(&mut self, entry: VirtAddr, stack: VirtAddr) {
        self.rip = entry.as_u64();
        self.rsp = stack.as_u64();
        self.rbp = 0;
    }
}

/// Realiza context switch entre duas tasks
///
/// # Safety
///
/// - Interrupções devem estar desabilitadas
/// - old e new devem ser ponteiros válidos
pub unsafe fn switch(old: &mut CpuContext, new: &CpuContext) {
    // Chamar assembly de switch
    context_switch_asm(
        old as *mut CpuContext as u64,
        new as *const CpuContext as u64,
    );
}

// Assembly implementation of context_switch_asm
// RDI = old (mut ptr), RSI = new (ptr)
// Struct offsets (CpuContext):
// 0:rbx, 8:rbp, 16:r12, 24:r13, 32:r14, 40:r15, 48:rsp, 56:rip
core::arch::global_asm!(
    r#"
.global context_switch_asm
context_switch_asm:
    // Save Generic Registers (Callee-saved)
    mov [rdi + 0x00], rbx
    mov [rdi + 0x08], rbp
    mov [rdi + 0x10], r12
    mov [rdi + 0x18], r13
    mov [rdi + 0x20], r14
    mov [rdi + 0x28], r15

    // Save Stack Pointer
    mov [rdi + 0x30], rsp

    // Save Instruction Pointer (Return Address)
    mov rax, [rsp]
    mov [rdi + 0x38], rax

    // --- Switch Point ---

    // Load New Context
    mov rbx, [rsi + 0x00]
    mov rbp, [rsi + 0x08]
    mov r12, [rsi + 0x10]
    mov r13, [rsi + 0x18]
    mov r14, [rsi + 0x20]
    mov r15, [rsi + 0x28]
    
    // Switch Stack
    mov rsp, [rsi + 0x30]

    // Return to new context
    ret
"#
);

extern "C" {
    fn context_switch_asm(old: u64, new: u64);
}
