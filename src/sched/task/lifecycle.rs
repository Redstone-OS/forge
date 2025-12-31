//! Cleanup de task

use crate::sys::types::Tid;

use super::entity::Task;
use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::pin::Pin;

/// Fila de tarefas mortas aguardando cleanup (reaper)
pub(crate) static ZOMBIES: Spinlock<VecDeque<Pin<Box<Task>>>> = Spinlock::new(VecDeque::new());

/// Adiciona tarefa à lista de zombies
pub fn add_zombie(task: Pin<Box<Task>>) {
    ZOMBIES.lock().push_back(task);
}

/// Finaliza a task atual
pub fn exit(code: i32) -> ! {
    crate::kinfo!("(Task) exit() chamado. Code=", code as u64);

    // Em teoria, atualizaríamos o código de saída no próprio task.
    // Mas para isso precisamos acesso mutável.
    // Como CURRENT está travado em run(), não podemos pegar mut ref fácil sem unsafe
    // ou sem suporte do scheduler.
    // O exit_current assume que a task vai embora.

    // Solução ideal: exit_current recebe o código.
    // Mas por enquanto, apenas chamamos o scheduler.
    crate::sched::core::exit_current();
}

/// Limpa recursos de uma task morta
///
/// Chamado pelo processo `init` ou `reaper` (ou idle loop) para liberar memória
/// de processos que já morreram.
pub fn cleanup(_tid: Tid) {
    let mut zombies = ZOMBIES.lock();

    // Procura e remove o zombie específico
    // TODO: Otimizar busca (Hashmap ou apenas pop se for FIFO)
    if let Some(pos) = zombies.iter().position(|t| t.tid == _tid) {
        let task = zombies.remove(pos).unwrap();
        crate::kinfo!(
            "(Lifecycle) Cleaning up zombie PID:",
            task.tid.as_u32() as u64
        );

        // Ao sair do escopo, `task` (Box<Task>) é dropado.
        // O `Drop` do Task/MemoryManager deve liberar as páginas.
        // Se a struct Task não tem Drop implementado para liberar frames,
        // perdemos memória (leak).
        // TODO: Implementar Drop for Task que libera kernel_stack/user_stack frames.
    }
}

/// Limpa todos os zumbis pendentes (útil para idle task chamar)
pub fn cleanup_all() {
    let mut zombies = ZOMBIES.lock();
    let count = zombies.len();
    if count > 0 {
        crate::kinfo!("(Lifecycle) Cleaning up all zombies. Count:", count as u64);
        zombies.clear(); // Dropa todos
    }
}
