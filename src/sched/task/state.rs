//! Estados de task

/// Estado de uma task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Recém criada, não executou ainda
    Created,
    /// Pronta para executar
    Ready,
    /// Executando em alguma CPU
    Running,
    /// Bloqueada esperando algo
    Blocked,
    /// Terminada, esperando cleanup
    Zombie,
    /// Morta, pode ser liberada
    Dead,
    /// Parada por sinal (SIGSTOP/Debugger)
    Stopped,
}

impl TaskState {
    /// Verifica se pode ser escalonada
    pub const fn is_runnable(self) -> bool {
        matches!(self, Self::Ready | Self::Running)
    }
}
