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

extern "C" {
    fn context_switch_asm(old: u64, new: u64);
}
