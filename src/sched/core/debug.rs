//! Ferramentas de Debug para o Scheduler

use super::runqueue::RUNQUEUE;
use super::scheduler::CURRENT;
use super::sleep_queue::SLEEP_QUEUE;
use crate::sched::task::lifecycle::ZOMBIES;
use crate::sched::task::TaskState;

/// Imprime o estado de todas as tarefas conhecidas no sistema
pub fn dump_tasks() {
    crate::ktrace!("--- [TRACE] GERENCIADOR DE TAREFAS ---");

    let mut total_tasks = 0u64;

    // 1. Task Atual (Running)
    if let Some(guard) = CURRENT.try_lock() {
        if let Some(ref task) = *guard {
            let tid = task.tid.as_u32();
            crate::ktrace!("  - Running TID:", tid as u64);
            total_tasks += 1;

            // Alerta se task em CURRENT não está Running (exceto idle)
            if task.state != TaskState::Running && tid != 0 {
                crate::kerror!(
                    "(Debug) BUG: Task em CURRENT não está Running! PID:",
                    tid as u64
                );
            }
        } else {
            crate::ktrace!("  - CURRENT: None");
        }
    } else {
        crate::ktrace!("  - CURRENT: [Locked]");
    }

    // 2. Ready Tasks
    if let Some(rq) = RUNQUEUE.try_lock() {
        crate::ktrace!("  - READY count:", rq.queue.len() as u64);
        for _task in &rq.queue {
            total_tasks += 1;
        }
    } else {
        crate::ktrace!("  - RUNQUEUE: [Locked]");
    }

    // 3. Sleeping Tasks
    if let Some(sq) = SLEEP_QUEUE.try_lock() {
        crate::ktrace!("  - SLEEPING count:", sq.len() as u64);
        for _ in sq.iter() {
            total_tasks += 1;
        }
    } else {
        crate::ktrace!("  - SLEEP_QUEUE: [Locked]");
    }

    // 4. Zombie Tasks
    if let Some(zombies) = ZOMBIES.try_lock() {
        crate::ktrace!("  - ZOMBIE count:", zombies.len() as u64);
        for _ in zombies.iter() {
            total_tasks += 1;
        }
    } else {
        crate::ktrace!("  - ZOMBIES: [Locked]");
    }

    // Alerta se perdemos tasks (esperamos 5: idle + 4 processos)
    // Nota: idle não conta pois está em CURRENT e não é contada separadamente
    if total_tasks < 4 && total_tasks > 0 {
        crate::kerror!("(Debug) ALERTA: Tasks desaparecendo! Total:", total_tasks);
    }

    crate::ktrace!("--- FIM DO DUMP ---");
}
