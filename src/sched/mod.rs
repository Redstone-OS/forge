//! # Scheduler (Sched)
//!
//! Gerenciamento de tarefas e escalonamento de CPU.
//!
//! ## Componentes
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                   SCHEDULER                         │
//! │  Policy → Runqueue → Pick next → Context Switch     │
//! └─────────────────────────────────────────────────────┘
//!                        ↑
//! ┌─────────────────────────────────────────────────────┐
//! │                     TASK                            │
//! │  Process/Thread → State → Context (CPU regs)        │
//! └─────────────────────────────────────────────────────┘
//!                        ↑
//! ┌─────────────────────────────────────────────────────┐
//! │                    EXEC                             │
//! │  ELF Loader → spawn() → setup stack → run           │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Estados de Task
//!
//! ```text
//! Created → Ready ⇄ Running → Blocked → Ready
//!                      ↓
//!                   Zombie → Reaped
//! ```

// =============================================================================
// TASK MANAGEMENT
// =============================================================================

/// Definição de Task (processo/thread)
pub mod task;

/// Contexto de CPU (registradores)
pub mod context;

/// Estados e transições
pub use task::{Task, TaskState, Tid};

// =============================================================================
// SCHEDULER
// =============================================================================

/// Algoritmos e runqueues
pub mod scheduler;

pub use scheduler::{schedule, yield_now};

// =============================================================================
// EXECUTION
// =============================================================================

/// Carregamento e execução de programas
pub mod exec;

pub use exec::{spawn, ExecError};

// =============================================================================
// SIGNALS
// =============================================================================

/// Sistema de sinais
pub mod signal;

// =============================================================================
// WAIT
// =============================================================================

/// Wait queues para bloqueio
pub mod wait;

pub use wait::WaitQueue;

// =============================================================================
// ASSEMBLY LINKAGE
// =============================================================================

// Importa assembly de context switch
core::arch::global_asm!(include_str!("../arch/x86_64/switch.s"));

extern "C" {
    /// Context switch em assembly
    pub fn context_switch(old_rsp: *mut u64, new_rsp: u64);
}

// =============================================================================
// TRAMPOLINES
// =============================================================================

/// Trampolim para userspace (Ring 0 → Ring 3)
#[naked]
#[no_mangle]
pub unsafe extern "C" fn user_entry_trampoline() {
    core::arch::asm!(
        "mov ax, 0x23", // USER_DATA_SEL | RPL 3
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "iretq",
        options(noreturn)
    );
}

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Inicializa o scheduler
pub fn init() {
    crate::kinfo!("(Sched) Inicializando scheduler...");
    scheduler::init();
    crate::kinfo!("(Sched) Scheduler inicializado");
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
