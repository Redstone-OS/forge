//! # Scheduler Core (Núcleo do Agendador)
//!
//! Implementação do agendador multi-core SMP.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    SCHEDULER CORE                       │
//! ├─────────────────────────────────────────────────────────┤
//! │                                                         │
//! │  per_cpu.rs ─────► cpu.rs (re-exports)                  │
//! │       │                                                 │
//! │       ├────────► scheduler.rs (schedule, yield, sleep)  │
//! │       │                                                 │
//! │       └────────► idle.rs (idle tasks per-CPU)           │
//! │                                                         │
//! └─────────────────────────────────────────────────────────┘
//! ```

/// Estruturas de dados per-CPU
pub mod per_cpu;

/// Ferramentas de diagnóstico
pub mod debug;

/// Pontos de entrada para novas tasks
pub mod entry;

/// Idle tasks per-CPU
pub mod idle;

/// Políticas de escalonamento
pub mod policy;

/// Runqueue (usada internamente por per_cpu)
pub mod runqueue;

/// Orquestrador de escalonamento
pub mod scheduler;

/// Fila de tasks dormindo
pub mod sleep_queue;

/// Context switch helpers
pub mod switch;

// =============================================================================
// RE-EXPORTS PRINCIPAIS
// =============================================================================

// Per-CPU
pub use per_cpu::{
    init_cpu, is_cpu_initialized, set_need_resched, should_resched, this_cpu_id, CpuState,
    LoadBalancer, PerCpuData, CPUS, MAX_CPUS,
};

// Idle
pub use idle::{
    get_idle_context_cpu, init_idle_task, init_idle_task_for_cpu, is_initialized,
    is_initialized_for_cpu, switch_to_idle_cpu,
};

// Scheduler
pub use scheduler::{
    current, enqueue, exit_current, init, pick_next, release_scheduler_lock, run, schedule,
    sleep_current, timer_tick, with_current, with_current_mut, yield_now,
};

// Debug
pub use debug::dump_tasks;

// Policy
pub use policy::SchedulingPolicy;

// Switch
pub use switch::prepare_and_switch_to;
