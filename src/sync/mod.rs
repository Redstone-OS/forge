//! # Synchronization Primitives
//!
//! Primitivas de controle de concorrência para o kernel.
//!
//! ## Primitivas Disponíveis
//!
//! | Primitiva    | Comportamento           | Uso                      |
//! |--------------|-------------------------|--------------------------|
//! | `Spinlock`   | Busy-wait + CLI         | IRQ handlers, seções < 1µs |
//! | `Mutex`      | Sleep (futuro)          | Seções longas            |
//! | `RwLock`     | N leitores OR 1 escritor| Dados muito lidos        |
//! | `Semaphore`  | Contador                | Pool de recursos         |
//! | `CondVar`    | Espera por condição     | Sincronização            |
//! | `Rcu`        | Lock-free reads         | Configs globais          |
//! | `Atomic*`    | Wrappers atômicos       | Contadores, flags        |
//!
//! ## Regras de Ouro
//!
//! 1. **IRQ = Spinlock**: Nunca use Mutex em interrupt handlers
//! 2. **Ordem de Aquisição**: Sempre adquira locks na mesma ordem
//! 3. **Hold Time Mínimo**: Segure locks pelo menor tempo possível

pub mod atomic;
pub mod condvar;
pub mod mutex;
pub mod rcu;
pub mod rwlock;
pub mod semaphore;
pub mod spinlock;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use atomic::{AtomicCell, AtomicCounter, AtomicFlag};
pub use condvar::CondVar;
pub use mutex::{Mutex, MutexGuard};
pub use rcu::{Rcu, RcuReadGuard};
pub use rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use semaphore::Semaphore;
pub use spinlock::{Spinlock, SpinlockGuard};
