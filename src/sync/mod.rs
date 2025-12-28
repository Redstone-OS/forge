//! # Synchronization Primitives
//!
//! Primitivas de sincronização para ambiente SMP.
//!
//! ## Hierarquia de Uso
//!
//! ```text
//! Spinlock   → Seções críticas curtas (não pode dormir)
//! Mutex      → Seções que podem bloquear (pode dormir)
//! RwLock     → Muitos leitores, poucos escritores
//! Semaphore  → Controle de recursos contáveis
//! CondVar    → Espera por condição
//! RCU        → Read-Copy-Update (leitura sem lock)
//! ```
//!
//! ## Regras
//!
//! - **Spinlock**: Usar apenas quando NÃO pode dormir (IRQ handlers)
//! - **Mutex**: Preferir para seções normais do kernel
//! - **Ordem de Lock**: Sempre adquirir na mesma ordem para evitar deadlock

// =============================================================================
// PRIMITIVAS BÁSICAS
// =============================================================================

/// Operações atômicas
pub mod atomic;

/// Spinlock (busy-wait, não dorme)
pub mod spinlock;

/// Mutex (pode bloquear thread)
pub mod mutex;

// =============================================================================
// PRIMITIVAS AVANÇADAS
// =============================================================================

/// Reader-Writer Lock
pub mod rwlock;

/// Semáforo (contagem de recursos)
pub mod semaphore;

/// Condition Variable
pub mod condvar;

/// Read-Copy-Update
pub mod rcu;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use atomic::AtomicCell;
pub use condvar::CondVar;
pub use mutex::{Mutex, MutexGuard};
pub use rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use semaphore::Semaphore;
pub use spinlock::{Spinlock, SpinlockGuard};
