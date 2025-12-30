//! Cleanup de task
//! Cleanup de task

use super::state::TaskState;
use crate::sys::types::Tid;

/// Finaliza a task atual
pub fn exit(code: i32) -> ! {
    crate::kinfo!("(Task) Tarefa encerrando com código:", code as u64);

    // TODO: Recuperar task atual
    // TODO: Definir exit code
    // TODO: Mudar estado para Zombie
    // TODO: Acordar parent (waitpid)
    // TODO: Chamar scheduler() para nunca mais voltar

    // Placeholder loop
    loop {
        crate::arch::Cpu::halt();
    }
}

/// Limpa recursos de uma task morta
pub fn cleanup(_tid: Tid) {
    // TODO: Liberar stack, pagetables, fd, etc.
}
