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

/// Salta diretamente para um contexto sem salvar o atual
/// Usado para a primeira task quando não há contexto anterior
///
/// # Safety
///
/// - Interrupções devem estar desabilitadas
/// - ctx deve ser um ponteiro válido
/// - Nunca retorna
pub unsafe fn jump_to_context(ctx: &CpuContext) -> ! {
    crate::ktrace!("(Switch) jump_to_context ENTRADA");
    crate::ktrace!("(Switch) ctx.rsp=", ctx.rsp);
    crate::ktrace!("(Switch) ctx.rip=", ctx.rip);

    // Verificar se a página do entry point está acessível
    let trapframe = ctx.rsp as *const u64;
    let user_entry = *trapframe;
    crate::ktrace!("(Switch) User entry point=", user_entry);

    // Verificar primeiros 4 bytes do código (deve ser B8 F3 00 00)
    let entry_ptr = user_entry as *const u8;
    let b0 = core::ptr::read_volatile(entry_ptr);
    let b1 = core::ptr::read_volatile(entry_ptr.add(1));
    let b8 = core::ptr::read_volatile(entry_ptr.add(8));
    let ba = core::ptr::read_volatile(entry_ptr.add(0xA));
    crate::ktrace!("(Switch) Code byte[0]=", b0 as u64);
    crate::ktrace!("(Switch) Code byte[1]=", b1 as u64);
    crate::ktrace!("(Switch) Code byte[8]=", b8 as u64);
    crate::ktrace!("(Switch) Code byte[A]=", ba as u64);

    // Verificar mapeamento de 0x400000
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));

    let vaddr = 0x400000u64;
    let pml4_idx = (vaddr >> 39) & 0x1FF;
    let pdpt_idx = (vaddr >> 30) & 0x1FF;
    let pd_idx = (vaddr >> 21) & 0x1FF;
    let pt_idx = (vaddr >> 12) & 0x1FF;

    let pml4_ptr = cr3 as *const u64;
    let pml4_entry = core::ptr::read_volatile(pml4_ptr.add(pml4_idx as usize));

    if pml4_entry & 1 != 0 {
        let pdpt_phys = pml4_entry & 0x000F_FFFF_FFFF_F000;
        let pdpt_ptr = pdpt_phys as *const u64;
        let pdpt_entry = core::ptr::read_volatile(pdpt_ptr.add(pdpt_idx as usize));

        if pdpt_entry & 1 != 0 && (pdpt_entry & (1 << 7)) == 0 {
            let pd_phys = pdpt_entry & 0x000F_FFFF_FFFF_F000;
            let pd_ptr = pd_phys as *const u64;
            let pd_entry = core::ptr::read_volatile(pd_ptr.add(pd_idx as usize));

            let is_huge_2mb = (pd_entry & (1 << 7)) != 0;
            if is_huge_2mb {
                crate::kerror!("(Switch) ERRO: 0x400000 ainda usa HUGE PAGE 2MB!");
            } else if pd_entry & 1 != 0 {
                let pt_phys = pd_entry & 0x000F_FFFF_FFFF_F000;
                let pt_ptr = pt_phys as *const u64;
                let pt_entry = core::ptr::read_volatile(pt_ptr.add(pt_idx as usize));
                let frame_phys = pt_entry & 0x000F_FFFF_FFFF_F000;
                crate::ktrace!("(Switch) Frame físico 0x400000=", frame_phys);
            }
        }
    }

    crate::ktrace!("(Switch) Chamando jump_to_context_asm...");
    jump_to_context_asm(ctx as *const CpuContext as u64);
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

    // Push RIP and ret to it
    mov rax, [rsi + 0x38]
    push rax
    ret

.global jump_to_context_asm
jump_to_context_asm:
    // RDI = ptr to CpuContext
    // Load all registers from context
    mov rbx, [rdi + 0x00]
    mov rbp, [rdi + 0x08]
    mov r12, [rdi + 0x10]
    mov r13, [rdi + 0x18]
    mov r14, [rdi + 0x20]
    mov r15, [rdi + 0x28]
    
    // Switch Stack
    mov rsp, [rdi + 0x30]

    // Jump to RIP directly (avoid push which writes to memory)
    mov rax, [rdi + 0x38]
    jmp rax

.global iretq_restore
iretq_restore:
    // Carregar segmentos de dados de usuário (RPL 3)
    // Isso é CRÍTICO: se DS/ES tiverem seletor de Kernel, pode causar #GP ou erros estranhos em User Mode
    mov ax, 0x1B  
    mov ds, ax
    mov es, ax
    swapgs
    iretq
"#
);

extern "C" {
    fn context_switch_asm(old: u64, new: u64);
    fn jump_to_context_asm(new: u64) -> !;
    pub fn iretq_restore() -> !;
}
