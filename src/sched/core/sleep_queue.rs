//! Sleep Queue - Gerencia tasks que estão dormindo
//!
//! Permite que tasks sejam bloqueadas por um tempo determinado e acordadas pelo timer.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::pin::Pin;

use crate::core::time::jiffies;
use crate::sched::task::{Task, TaskState};
use crate::sync::Spinlock;

/// Fila global de tasks dormindo
pub static SLEEP_QUEUE: Spinlock<VecDeque<Pin<Box<Task>>>> = Spinlock::new(VecDeque::new());

/// Verifica se há tasks que devem acordar e as move para a RunQueue
pub fn check_sleep_queue() {
    let now = jiffies::get_jiffies();
    let mut sleep_queue = SLEEP_QUEUE.lock();

    // Usamos um contador manual para evitar problemas ao remover itens enquanto iteramos
    let mut i = 0;
    while i < sleep_queue.len() {
        let should_wake = if let Some(wake_at) = sleep_queue[i].wake_at {
            now >= wake_at
        } else {
            false
        };

        if should_wake {
            // Remove da fila de sleep
            if let Some(mut task) = sleep_queue.remove(i) {
                crate::kinfo!("(Sleep) Acordando task PID:", task.tid.as_u32() as u64);

                // Limpa o wake_at e marca como pronta
                task.wake_at = None;
                task.state = TaskState::Ready;

                // Devolve para a RunQueue global
                crate::sched::core::enqueue(task);

                // Não incrementa i pois o próximo item agora está nesta posição
                continue;
            }
        }
        i += 1;
    }
}

pub fn add_task(task: Pin<Box<Task>>) {
    SLEEP_QUEUE.lock().push_back(task);
}
