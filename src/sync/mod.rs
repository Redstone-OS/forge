//! Primitivas de Sincronização
//!
//! Contém Spinlocks, Mutexes, Semáforos e Atomics.

pub mod atomic;
pub mod condvar;
pub mod mutex;
pub mod rcu;
pub mod rwlock;
pub mod semaphore;
pub mod spinlock;

pub use atomic::{AtomicCell, AtomicCounter, AtomicFlag};
pub use mutex::Mutex;
pub use rwlock::RwLock;
pub use semaphore::Semaphore;
pub use spinlock::Spinlock;
