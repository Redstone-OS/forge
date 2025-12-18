//! Scheduler Round-Robin
//!
//! Scheduler simples com quantum fixo e preempção por timer.

use super::process::{Pid, ProcessState, PROCESS_MANAGER};
use spin::Mutex;

/// Scheduler
pub struct Scheduler {
    quantum_ticks: u64,
    current_ticks: u64,
}

impl Scheduler {
    /// Cria novo scheduler
    pub const fn new() -> Self {
        Self {
            quantum_ticks: 1, // 10ms @ 100Hz
            current_ticks: 0,
        }
    }

    /// Chamado a cada timer tick
    /// Retorna true se quantum expirou
    pub fn tick(&mut self) -> bool {
        self.current_ticks += 1;

        if self.current_ticks >= self.quantum_ticks {
            self.current_ticks = 0;
            true // Quantum expirado
        } else {
            false
        }
    }

    /// Escolhe próximo processo (round-robin)
    /// Retorna PID do próximo processo
    pub fn schedule() -> Option<Pid> {
        let mut pm = PROCESS_MANAGER.lock();

        // Se há processo atual Running, marcar como Ready e mover para fim
        if let Some(current_pid) = pm.current_pid {
            if let Some(current) = pm.get_mut(current_pid) {
                if current.state == ProcessState::Running {
                    current.state = ProcessState::Ready;
                }
            }
        }

        // Pegar próximo processo Ready
        if let Some(next) = pm.next_ready() {
            next.state = ProcessState::Running;
            let pid = next.pid;
            pm.current_pid = Some(pid);
            Some(pid)
        } else {
            None
        }
    }
}

/// Scheduler global
pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
