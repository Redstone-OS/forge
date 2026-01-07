//! Ferramentas de Debug para o Scheduler Multi-Core

use super::per_cpu::{CPUS, MAX_CPUS};
use super::sleep_queue::SLEEP_QUEUE;
use crate::sched::task::lifecycle::ZOMBIES;
use crate::sched::task::TaskState;

/// Imprime o estado de todas as tarefas no sistema
pub fn dump_tasks() {
    crate::ktrace!("--- [TRACE] SCHEDULER MULTI-CORE ---");

    let mut total_tasks = 0u64;
    let mut total_ready = 0u64;

    // 1. Dump de cada CPU
    for cpu_id in 0..MAX_CPUS {
        if let Some(guard) = CPUS[cpu_id].try_lock() {
            if let Some(ref cpu_data) = *guard {
                crate::ktrace!("  CPU:");

                // Current task
                if let Some(ref task) = cpu_data.current {
                    let tid = task.tid.as_u32();
                    crate::ktrace!("    - Running TID:", tid as u64);
                    total_tasks += 1;

                    if task.state != TaskState::Running && tid != 0 {
                        crate::kerror!("(Debug) BUG: Task não está Running!");
                    }
                } else {
                    crate::ktrace!("    - Current: None");
                }

                // Runqueue local
                let rq_len = cpu_data.runqueue_len();
                crate::ktrace!("    - Ready:", rq_len as u64);
                total_ready += rq_len as u64;
                total_tasks += rq_len as u64;

                // Idle task
                if cpu_data.idle_task.is_some() {
                    crate::ktrace!("    - Idle: OK");
                }
            }
        }
    }

    // 2. Sleeping Tasks
    if let Some(sq) = SLEEP_QUEUE.try_lock() {
        crate::ktrace!("  SLEEPING count:", sq.len() as u64);
        total_tasks += sq.len() as u64;
    } else {
        crate::ktrace!("  SLEEP_QUEUE: [Locked]");
    }

    // 3. Zombie Tasks
    if let Some(zombies) = ZOMBIES.try_lock() {
        crate::ktrace!("  ZOMBIE count:", zombies.len() as u64);
        total_tasks += zombies.len() as u64;
    } else {
        crate::ktrace!("  ZOMBIES: [Locked]");
    }

    crate::ktrace!("  TOTAL TASKS:", total_tasks);
    crate::ktrace!("--- FIM DO DUMP ---");
}
