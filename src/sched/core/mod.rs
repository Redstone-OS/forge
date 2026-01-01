//! # Scheduler Core (Núcleo do Agendador)
//!
//! Este módulo contém a implementação fundamental do agendador do RedstoneOS.
//! Ele é responsável pela mecânica de baixo nível necessária para alternar
//! a execução entre diferentes tarefas, gerenciar filas e lidar com ociosidade.
//!
//! ## Componentes:
//! - **Gerenciamento:** `scheduler.rs` coordena o ciclo de vida do agendamento.
//! - **Estado de Hardware:** `switch.rs` e `entry.rs` lidam com registradores e saltos.
//! - **Filas:** `runqueue.rs` (prontos) e `sleep_queue.rs` (dormindo).
//! - **Ociosidade:** `idle.rs` gerencia o consumo de CPU quando não há trabalho.

/// Gerenciamento de dados específicos por CPU e balanceamento de carga (SMP Ready).
pub mod cpu;

/// Ferramentas de diagnóstico e dump do estado interno do agendador.
pub mod debug;

/// Pontos de entrada e trampolins em assembly para novas tarefas.
pub mod entry;

/// Lógica de espera e baixo consumo de energia quando não há tarefas prontas.
pub mod idle;

/// Definições de políticas de escalonamento (Round Robin, Prioridade, etc).
pub mod policy;

/// Implementação da fila de tarefas prontas para execução (Ready).
pub mod runqueue;

/// O orquestrador central que decide quando e como trocar de tarefa.
pub mod scheduler;

/// Gerenciador de tarefas que aguardam um determinado tempo (Sleep).
pub mod sleep_queue;

/// Mecânica de baixo nível para salvar e restaurar o contexto da CPU.
pub mod switch;

// Re-exportações de tipos e funções essenciais para simplificar o uso pelo resto do kernel.
pub use debug::dump_tasks;
pub use idle::{init_idle_task, is_initialized as is_idle_initialized, IDLE_TASK};
pub use policy::SchedulingPolicy;
pub use scheduler::{
    current, enqueue, exit_current, init, pick_next, release_scheduler_lock, run, schedule,
    sleep_current, yield_now, CURRENT,
};
pub use switch::prepare_and_switch_to;
