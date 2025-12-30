//! Fila de tasks prontas

use super::super::task::Task;
use crate::sync::Spinlock;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::pin::Pin;

/// Fila de execução (FIFO simples por enquanto).
/// TODO: Implementar Multi-level Feedback Queue ou Array de Prioridades.
pub struct RunQueue {
    queue: VecDeque<Pin<Box<Task>>>,
}

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Adiciona task à fila
    pub fn push(&mut self, task: Pin<Box<Task>>) {
        self.queue.push_back(task);
    }

    /// Remove próxima task (FIFO)
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

/// Runqueue global (TODO: per-CPU)
pub static RUNQUEUE: Spinlock<RunQueue> = Spinlock::new(RunQueue::new());
