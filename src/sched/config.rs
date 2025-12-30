//! Constantes de configuração do Scheduler

/// Prioridade mínima (IDLE)
pub const PRIORITY_MIN: u8 = 0;

/// Prioridade padrão para processos de usuário
pub const PRIORITY_DEFAULT: u8 = 128;

/// Prioridade máxima (Realtime/Kernel)
pub const PRIORITY_MAX: u8 = 255;

/// Prioridade da tarefa Idle
pub const PRIORITY_IDLE: u8 = 0; // Geralmente 0 é a menor

/// Tamanho padrão da Stack de Kernel (em bytes)
pub const KERNEL_STACK_SIZE: usize = 65536; // 64KB

/// Quantum padrão (Timeslice) em ticks do timer
pub const DEFAULT_QUANTUM: u64 = 10;

/// Tamanho padrão da Stack de Usuário (em bytes) - 2MB
pub const USER_STACK_SIZE: usize = 2 * 1024 * 1024;

/// Número máximo de tarefas no sistema (segurança)
pub const MAX_TASKS: usize = 65536;

/// Intervalo de balanceamento de carga (em ticks) - Placeholder para SMP
pub const LOAD_BALANCE_INTERVAL: u64 = 100;
