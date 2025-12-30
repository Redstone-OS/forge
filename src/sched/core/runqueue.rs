//! Fila de tasks prontas

use super::super::task::Task;
use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::pin::Pin;

/// Fila de execução global (Single Core).
///
/// Armazena as tarefas que estão no estado `Ready` e aguardam tempo de CPU.
/// Atualmente implementa uma política FIFO (Round Robin simples).
pub struct RunQueue {
    queue: VecDeque<Pin<Box<Task>>>,
}

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Adiciona task à fila (final da fila)
    pub fn push(&mut self, task: Pin<Box<Task>>) {
        self.queue.push_back(task);
    }

    /// Remove próxima task (início da fila)
    pub fn pop(&mut self) -> Option<Pin<Box<Task>>> {
        self.queue.pop_front()
    }

    /// Número de tasks na fila
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Verifica se está vazia
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Runqueue global
///
/// Em implementações futuras SMP, isso pode virar um array `[RunQueue; MAX_CPUS]`
/// ou ser movido para dentro da struct `Cpu`.
pub static RUNQUEUE: Spinlock<RunQueue> = Spinlock::new(RunQueue::new());
