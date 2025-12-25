//! Módulo de Agendamento (Scheduler).

pub mod context;
pub mod scheduler;
pub mod task;
pub mod test;

// Importa o assembly de troca de contexto
core::arch::global_asm!(include_str!("../arch/x86_64/switch.s"));

extern "C" {
    /// Função assembly definida em switch.s
    pub fn context_switch(old_rsp_ptr: *mut u64, new_rsp: u64);
}

/// Trampolim para pular para Userspace.
#[naked]
pub unsafe extern "C" fn user_entry_trampoline() {
    core::arch::asm!(
        // Restaurar segmentos de dados de usuário (Ring 3)
        "mov ax, 0x23", // USER_DATA_SEL (0x20) | RPL 3
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        // A stack já tem [RIP, CS, RFLAGS, RSP, SS] empilhados
        // Executar IRETQ para trocar de Ring 0 -> Ring 3
        "iretq",
        options(noreturn)
    );
}
