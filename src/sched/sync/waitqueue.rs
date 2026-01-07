//! Wait queues para bloqueio e sincronização
//!
//! Permite que threads durmam aguardando eventos e sejam acordadas posteriormente.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::pin::Pin;

use crate::sched::core::per_cpu::{this_cpu_id, CPUS};
use crate::sched::task::{Task, TaskState};
use crate::sync::Spinlock;

/// Wait queue - fila de tarefas bloqueadas aguardando um evento.
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
    pub fn wait(&self) {
        crate::arch::Cpu::disable_interrupts();

        let cpu_id = this_cpu_id();

        // 1. Pegar a task atual para bloquear
        let task = {
            let mut guard = CPUS[cpu_id].lock();
            if let Some(ref mut cpu_data) = *guard {
                if let Some(mut task) = cpu_data.current.take() {
                    unsafe { Pin::get_unchecked_mut(task.as_mut()) }.state = TaskState::Blocked;
                    task
                } else {
                    crate::kerror!("(WaitQueue) wait: sem task atual!");
                    drop(guard);
                    crate::arch::Cpu::enable_interrupts();
                    return;
                }
            } else {
                crate::kerror!("(WaitQueue) wait: CPU não inicializada!");
                crate::arch::Cpu::enable_interrupts();
                return;
            }
        };

        // 2. Adicionar à fila de espera
        self.waiters.lock().push_back(task);

        // 3. Chamar schedule para escolher próxima task
        crate::sched::core::scheduler::schedule();

        crate::arch::Cpu::enable_interrupts();
    }

    /// Acorda uma thread desta fila, movendo-a para a RunQueue.
    pub fn wake_one(&self) -> bool {
        let mut waiters = self.waiters.lock();
        if let Some(mut task) = waiters.pop_front() {
            task.set_ready();
            crate::sched::core::enqueue(task);
            true
        } else {
            false
        }
    }

    /// Acorda todas as threads desta fila.
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
