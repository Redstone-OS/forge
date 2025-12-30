//! Entrega de Sinais

use super::{SIGCONT, SIGKILL, SIGSTOP};
use crate::sched::task::Task;
use crate::sched::task::TaskState;

/// Verifica e processa sinais pendentes para a tarefa atuaL.
///
/// Deve ser chamado no retorno de interrupções/syscalls (antes de voltar pro user).
pub fn process_pending_signals(task: &mut Task) {
    if task.pending_signals == 0 {
        return;
    }

    // Iterar sinais (bitmask)
    for sig in 1..32 {
        if (task.pending_signals & (1 << sig)) != 0 {
            // Limpar bit
            task.pending_signals &= !(1 << sig);

            deliver_signal(task, sig);
        }
    }
}

/// Entrega um sinal específico
fn deliver_signal(task: &mut Task, signum: i32) {
    // TODO: Consultar SignalHandlers da Task (quando implementado na struct)
    // Por enquanto, implementação hardcoded básica

    match signum {
        SIGKILL => {
            crate::kinfo!("(Signal) Task recebeu SIGKILL. Terminando.");
            // Força exit (simulado)
            // Na prática, marcaria flag de exit e schedule() limparia depois.
            // Impossível chamar exit daqui se não tiver ownership do scheduler.
            // Apenas marcamos o exit_code e estado por enquanto.
            task.state = TaskState::Zombie;
            task.exit_code = Some(128 + SIGKILL);
        }
        SIGSTOP => {
            crate::kinfo!("(Signal) Task recebeu SIGSTOP. Parando.");
            task.state = TaskState::Stopped;
            // Scheduler vai precisar ignorar tarefas Stopped
        }
        SIGCONT => {
            if task.state == TaskState::Stopped {
                task.state = TaskState::Ready;
            }
        }
        _ => {
            crate::kinfo!(
                "(Signal) Sinal ignorado (não implementado handler):",
                signum as u64
            );
        }
    }
}
