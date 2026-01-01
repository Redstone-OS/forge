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
    // Ao entrar aqui (via context_switch_asm 'push rax; ret'):
    // RSP está no topo da área reservada (8 bytes acima do TrapFrame).
    // O 'push rax' consumiu os 8 bytes e 'ret' restaurou RSP ao topo da reserva.
    // Precisamos descer 48 bytes (40 do TF + 8 da Reserva) para apontar ao TrapFrame real.
    sub rsp, 48
    
    // Agora RSP aponta para o ExceptionStackFrame (TrapFrame) preparado pelo loader.
    // Layout: [RIP, CS, RFLAGS, RSP, SS]
    
    // Precisamos chamar release_scheduler_lock (função Rust).
    // Ela pode usar stack. Então precisamos preservar nosso ponteiro de TrapFrame.
    // Vamos usar um registrador callee-saved (r12) para guardar o RSP original.
    mov r12, rsp
    
    // Alinhar stack para chamada (System V ABI requer alinhamento de 16 bytes)
    and rsp, -16
    
    // Chamar função de desbloqueio
    call release_scheduler_lock
    
    // Restaurar RSP original (apontando pro TrapFrame)
    mov rsp, r12
    
    // Limpar registradores de propósito geral (Segurança e Ambiente limpo para init)
    xor rax, rax
    xor rbx, rbx
    xor rcx, rcx
    xor rdx, rdx
    xor rsi, rsi
    xor rdi, rdi
    xor r8, r8
    xor r9, r9
    xor r10, r10
    xor r11, r11
    // R12 foi usado por nós, R13-R15 já estão em estado indefinido ou CpuContext
    xor r12, r12

    // Pular para o restaurador de contexto
    // IMPORTANTE: JMP, não CALL, pois não queremos voltar aqui e não queremos sujar a stack
    jmp iretq_restore
"#
);

extern "C" {
    pub fn user_entry_stub();
}
