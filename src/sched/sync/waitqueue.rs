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

        // 1. Pegar a task atual para bloquear
        let (task, old_ctx_ptr) = {
            let mut current_guard = CURRENT.lock();
            if let Some(mut task) = current_guard.take() {
                // Marcar como bloqueada
                unsafe { Pin::get_unchecked_mut(task.as_mut()) }.state = TaskState::Blocked;

                // Obter ponteiro do contexto para o switch
                let ctx_ptr =
                    unsafe { &mut Pin::get_unchecked_mut(task.as_mut()).context as *mut _ };

                // Devolve a task e o ponteiro
                (task, ctx_ptr)
            } else {
                crate::kerror!("(WaitQueue) wait called without current task!");
                crate::arch::Cpu::enable_interrupts();
                return;
            }
        };

        // 2. Adicionar à fila de espera (agora detemos a ownership da task)
        self.waiters.lock().push_back(task);

        // 3. Escolher a próxima task e trocar de contexto
        // IMPORTANTE: precisamos pegar o lock do CURRENT de volta para o prepare_and_switch_to
        let current_guard = CURRENT.lock();
        if let Some(next) = crate::sched::core::pick_next() {
            unsafe {
                crate::sched::core::prepare_and_switch_to(next, Some(old_ctx_ptr), current_guard);
            }
        } else {
            // Se não houver próxima, o sistema entra em Idle
            // O enter_idle_loop vai salvar o contexto em old_ctx_ptr e aguardar.
            drop(current_guard);
            unsafe {
                crate::sched::core::enter_idle_loop(Some(old_ctx_ptr));
            }
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
