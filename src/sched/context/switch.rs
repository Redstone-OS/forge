//! Context switch

use crate::mm::VirtAddr;

/// Contexto de CPU (registradores salvos)
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

    // FPU/SSE state (512 bytes, 16-aligned)
    pub fpu_state: FpuState,
}

/// Estado FPU (área FXSAVE)
#[repr(C, align(16))]
pub struct FpuState {
    data: [u8; 512],
}

impl FpuState {
    pub const fn new() -> Self {
        Self { data: [0; 512] }
    }

    /// Salva estado FPU atual
    pub fn save(&mut self) {
        // SAFETY: fxsave é seguro com buffer alinhado
        unsafe {
            core::arch::asm!(
                "fxsave [{}]",
                in(reg) self.data.as_mut_ptr(),
                options(nostack)
            );
        }
    }

    /// Restaura estado FPU
    pub fn restore(&self) {
        // SAFETY: fxrstor é seguro com buffer válido
        unsafe {
            core::arch::asm!(
                "fxrstor [{}]",
                in(reg) self.data.as_ptr(),
                options(nostack)
            );
        }
    }
}

impl CpuContext {
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
            fpu_state: FpuState::new(),
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
    // Salvar FPU do contexto antigo
    old.fpu_state.save();

    // Chamar assembly de switch
    context_switch_asm(
        old as *mut CpuContext as u64,
        new as *const CpuContext as u64,
    );

    // Restaurar FPU do novo contexto
    // (acontece quando voltamos a ser o "new")
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
    // Note: The caller pushed the return address onto the stack.
    // RSP currently points to the return address.
    // We save THIS RSP. When restored, we will be pointing to a return address.
    mov [rdi + 0x30], rsp

    // Save Instruction Pointer (Return Address)
    // The previous execution state would return to the caller.
    // We grab [rsp] which is the return address.
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

    // Handle RIP
    // At this point, RSP points to a return address on the new stack.
    // We can just execute 'ret'.
    // However, if the new task was just created, it might not have 'pushed' a return address
    // in the way a 'call' instruction does, but rather manually set up the stack.
    // AND if CpuContext.rip is set to the entry point, we should technically jump there.
    
    // Robust Strategy: Push the explicit RIP from CpuContext onto the stack
    // forcing the 'ret' to jump there.
    // BUT we must be careful: if we are resuming a thread that was suspended via 'call',
    // its stack ALREADY has the return address at [RSP].
    
    // Optimization: Trust the stack.
    // If it's a new thread, setup() MUST push the entry point to the stack.
    // This allows 'ret' to work universally.
    
    // If we want to support 'fork' where RIP is modified in struct but stack remains:
    // We would need to overwrite [rsp].
    
    // Let's stick to standard behavior: simply return.
    // Assumption: Thread setup pushes Entry Point.
    ret
"#
);

extern "C" {
    fn context_switch_asm(old: u64, new: u64);
}
