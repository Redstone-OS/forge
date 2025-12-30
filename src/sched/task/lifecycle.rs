//! Cleanup de task
//! Cleanup de task

use crate::sys::types::Tid;

use super::entity::Task;
use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::pin::Pin;

/// Fila de tarefas mortas aguardando cleanup (reaper)
static ZOMBIES: Spinlock<VecDeque<Pin<Box<Task>>>> = Spinlock::new(VecDeque::new());

/// Adiciona tarefa à lista de zombies
pub fn add_zombie(task: Pin<Box<Task>>) {
    ZOMBIES.lock().push_back(task);
}

/// Finaliza a task atual
pub fn exit(code: i32) -> ! {
    // Apenas loga e chama o scheduler para matar a task.
    // A lógica de limpeza real acontece em sched::core::exit_current
    // que manipula o lock do scheduler.
    crate::kinfo!("(Task) exit() chamado. Code=", code as u64);

    // Nota: Em um sistema completo, aqui setaríamos o exit_code na struct Task
    // ANTES de chamar exit_current, mas como Task está dentro de um Lock no scheduler,
    // precisamos passar essa info ou adquirir o lock aqui.
    // Por simplicidade, assumimos que exit_current lidará com a transição.

    crate::sched::core::exit_current();
}

/// Limpa recursos de uma task morta
pub fn cleanup(_tid: Tid) {
    // TODO: Liberar stack, pagetables, fd, etc.
}
