//! Ferramentas de Debug para o Scheduler

use super::runqueue::RUNQUEUE;
use super::scheduler::CURRENT;
use super::sleep_queue::SLEEP_QUEUE;
use crate::sched::task::lifecycle::ZOMBIES;

/// Imprime o estado de todas as tarefas conhecidas no sistema
pub fn dump_tasks() {
    crate::ktrace!("--- [TRACE] GERENCIADOR DE TAREFAS: LISTA COMPLETA ---");

    // 1. Task Atual (Running)
    if let Some(guard) = CURRENT.try_lock() {
        if let Some(ref task) = *guard {
            crate::ktrace!("  - Running TID:", task.tid.as_u32() as u64);
            crate::ktrace!("    State:", task.state as u32 as u64);
        } else {
            crate::ktrace!("  - CURRENT: None");
        }
    } else {
        crate::ktrace!("  - CURRENT: [Locked]");
    }

    // 2. Ready Tasks
    if let Some(rq) = RUNQUEUE.try_lock() {
        crate::ktrace!("  - READY Tasks count:", rq.queue.len() as u64);
        for task in &rq.queue {
            crate::ktrace!("    -> TID:", task.tid.as_u32() as u64);
        }
    } else {
        crate::ktrace!("  - RUNQUEUE: [Locked]");
    }

    // 3. Sleeping Tasks
    if let Some(sq) = SLEEP_QUEUE.try_lock() {
        crate::ktrace!("  - SLEEPING Tasks count:", sq.len() as u64);
        for task in sq.iter() {
            crate::ktrace!("    -> TID:", task.tid.as_u32() as u64);
        }
    } else {
        crate::ktrace!("  - SLEEP_QUEUE: [Locked]");
    }

    // 4. Zombie Tasks
    if let Some(zombies) = ZOMBIES.try_lock() {
        crate::ktrace!("  - ZOMBIE Tasks count:", zombies.len() as u64);
        for task in zombies.iter() {
            crate::ktrace!("    -> TID:", task.tid.as_u32() as u64);
        }
    } else {
        crate::ktrace!("  - ZOMBIES: [Locked]");
    }

    crate::ktrace!("--- [TRACE] FIM DO DUMP ---");
}
