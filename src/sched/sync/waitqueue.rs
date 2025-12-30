//! Wait queues para bloqueio e sincronização
//!
//! Permite que threads durmam aguardando eventos e sejam acordadas posteriormente.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::pin::Pin;

use crate::sched::core::CURRENT;
use crate::sched::task::{Task, TaskState};
use crate::sync::Spinlock;

/// Wait queue - fila de tarefas bloqueadas aguardando um evento.
///
/// Diferente da implementação anterior, armazenamos a `Task` inteira (ownership),
/// retirando-a do agendador. Ao acordar, devolvemos para a `RunQueue`.
pub struct WaitQueue {
    waiters: Spinlock<VecDeque<Pin<Box<Task>>>>,
}

impl WaitQueue {
    /// Cria nova waitqueue vazia
    pub const fn new() -> Self {
        Self {
            waiters: Spinlock::new(VecDeque::new()),
        }
    }

    /// Bloqueia a thread atual e a coloca nesta fila de espera.
    ///
    /// O scheduler escolherá outra tarefa para rodar.
    pub fn wait(&self) {
        crate::arch::Cpu::disable_interrupts();

        // 1. Pegar a task atual (takeownership do CURRENT)
        // Precisamos do lock do scheduler para fazer isso atomicamente em relação a interrupts
        let task = {
            let mut current_guard = CURRENT.lock();
            current_guard.take() // Remove de CURRENT
        };

        if let Some(mut task) = task {
            // 2. Mudar estado para Blocked
            task.state = TaskState::Blocked;

            // 3. Adicionar à fila de espera
            self.waiters.lock().push_back(task);

            // 4. Chamar schedule() para rodar a próxima
            // Como CURRENT está None (take), o schedule() vai pegar a próxima do RunQueue.
            // O lock do CURRENT já foi solto.
            crate::sched::core::schedule();
        } else {
            // Se wait() for chamado sem task rodando (ex: boot), panic ou ignorar.
            crate::kerror!("(WaitQueue) wait called without current task!");
        }

        crate::arch::Cpu::enable_interrupts();
    }

    /// Acorda uma thread desta fila, movendo-a para a RunQueue.
    ///
    /// Retorna true se acordou alguém.
    pub fn wake_one(&self) -> bool {
        let mut waiters = self.waiters.lock();
        if let Some(mut task) = waiters.pop_front() {
            // 1. Mudar estado para Ready
            task.set_ready();

            // 2. Devolver para RunQueue
            crate::sched::core::enqueue(task);
            true
        } else {
            false
        }
    }

    /// Acorda todas as threads desta fila.
    ///
    /// Retorna número de threads acordadas.
    pub fn wake_all(&self) -> usize {
        let mut waiters = self.waiters.lock();
        let mut count = 0;
        while let Some(mut task) = waiters.pop_front() {
            task.set_ready();
            crate::sched::core::enqueue(task);
            count += 1;
        }
        count
    }
}
