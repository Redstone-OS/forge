//!Entry points para novas tasks
//! Trampolim para novas tarefas de usuário
//!
//! Chamado via `jump_to_context` (RIP aponta para cá configurado no loader).
//! Responsável por:
//! 1. Liberar o lock do scheduler (que foi herdado da task anterior).
//! 2. Habilitar interrupções (via iretq).
//! 3. Executar iretq para pular para ring 3.
//!
//! IMPLEMENTAÇÃO EM ASSEMBLY NECESSÁRIA!
//! Uma função Rust normal alteraria o RSP, corrompendo o TrapFrame que está
//! no topo da task.context.rsp. O `iretq` exige que o RSP aponte EXATAMENTE
//! para o TrapFrame.
core::arch::global_asm!(
    r#"
.global user_entry_stub
.extern release_scheduler_lock
.extern iretq_restore

user_entry_stub:
    // Ao entrar aqui:
    // RSP aponta para o ExceptionStackFrame (TrapFrame) preparado pelo loader.
    // [RIP, CS, RFLAGS, RSP, SS]
    
    // Precisamos chamar release_scheduler_lock (função Rust).
    // Ela pode usar stack. Então precisamos preservar nosso ponteiro de TrapFrame.
    // Vamos usar um registrador callee-saved (rbx ou r12-r15) para guardar o RSP original.
    mov r12, rsp
    
    // Alinhar stack para chamada (System V ABI requer alinhamento de 16 bytes)
    and rsp, -16
    
    // Chamar função de desbloqueio
    call release_scheduler_lock
    
    // Restaurar RSP original (apontando pro TrapFrame)
    mov rsp, r12
    
    // Pular para o restaurador de contexto
    // IMPORTANTE: JMP, não CALL, pois não queremos voltar aqui e não queremos sujar a stack
    jmp iretq_restore
"#
);

extern "C" {
    pub fn user_entry_stub();
}
