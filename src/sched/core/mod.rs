//! Scheduler principal
//!
//! Este módulo contém os componentes centrais do agendador do Kernel.
//! A lógica de escalonamento reside em `scheduler.rs`.

pub mod cpu;
pub mod entry;
pub mod policy;
pub mod runqueue;
pub mod scheduler;

pub use policy::SchedulingPolicy;

// Re-exporta funções públicas do scheduler para facilitar acesso
pub use scheduler::{
    current, enqueue, exit_current, init, pick_next, release_scheduler_lock, run, schedule,
    yield_now, CURRENT,
};
