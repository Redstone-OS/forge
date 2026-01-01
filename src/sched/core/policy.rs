//! Algoritmos de escalonamento

/// Políticas de escalonamento suportadas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingPolicy {
    /// Round Robin (Timeslicing) - Padrão para processos normais
    RoundRobin,
    /// First-In First-Out - Realtime
    Fifo,
    /// Deadline - Hard Realtime (Futuro)
    Deadline,
}

// use crate::sched::config::{PRIORITY_DEFAULT, PRIORITY_MAX, PRIORITY_MIN};

impl Default for SchedulingPolicy {
    fn default() -> Self {
        Self::RoundRobin
    }
}
