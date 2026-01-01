//! Contabilidade de Recursos (Accounting)
//!
//! Este módulo é responsável por rastrear o consumo de recursos por cada tarefa,
//! incluindo tempo de CPU, trocas de contexto e estatísticas de execução.

/// Estatísticas de uso de recursos de uma tarefa
#[derive(Debug, Clone, Copy, Default)]
pub struct Accounting {
    /// Tempo total de CPU consumido (em ticks do sistema ou nanossegundos)
    pub total_cpu_time: u64,

    /// Tempo consumido em modo usuário (se suportado pelo hardware/timer)
    pub user_cpu_time: u64,

    /// Tempo consumido em modo kernel
    pub kernel_cpu_time: u64,

    /// Timestamp (em ticks) da última vez que a tarefa começou a executar.
    /// Usado para calcular o delta quando ela perde a CPU.
    pub last_start_time: u64,

    /// Número de trocas de contexto voluntárias (ex: yield, esperar I/O)
    pub voluntary_switches: u64,

    /// Número de trocas de contexto involuntárias (ex: preempção por quantum expirado)
    pub involuntary_switches: u64,

    /// Quantum restante para esta task nesta fatia de tempo (em ticks)
    pub quantum_left: u64,
}

impl Accounting {
    /// Cria uma nova estrutura de contabilidade zerada
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra o início da execução (chamado quando a task ganha a CPU)
    pub fn start_exec(&mut self, now: u64) {
        self.last_start_time = now;
        self.reset_quantum();
    }

    /// Reinicia o quantum da task
    pub fn reset_quantum(&mut self) {
        // TODO: No futuro, o quantum deve ser calculado com base na prioridade da task.
        // Tasks com maior prioridade deveriam receber fatias de tempo maiores.
        self.quantum_left = crate::sched::config::DEFAULT_QUANTUM;
    }

    /// Registra o fim da execução (chamado quando a task perde a CPU)
    /// Retorna o tempo executado nesta fatia.
    pub fn end_exec(&mut self, now: u64) -> u64 {
        if now >= self.last_start_time {
            let delta = now - self.last_start_time;
            self.total_cpu_time += delta;
            // Por padrão, assumimos tudo como kernel time se não tivermos tracking fino ainda
            // Futuramente podemos dividir baseados no RIP salvo
            self.kernel_cpu_time += delta;
            delta
        } else {
            // Relógio voltou no tempo? Ignora.
            0
        }
    }

    /// Incrementa contadores de troca de contexto
    pub fn account_switch(&mut self, voluntary: bool) {
        if voluntary {
            self.voluntary_switches += 1;
        } else {
            self.involuntary_switches += 1;
        }
    }
}
