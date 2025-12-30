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

pub const PRIORITY_MIN: u8 = 0;
pub const PRIORITY_DEFAULT: u8 = 128;
pub const PRIORITY_MAX: u8 = 255;
pub const PRIORITY_IDLE: u8 = 255;

impl Default for SchedulingPolicy {
    fn default() -> Self {
        Self::RoundRobin
    }
}
