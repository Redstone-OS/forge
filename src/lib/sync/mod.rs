//! Primitivas de Sincronização
//!
//! Locks, semáforos, condition variables, etc.
//!
//! # TODOs
//! - TODO(prioridade=média, versão=v1.0): Migrar de sync/

pub mod mutex;
pub mod spinlock;
pub mod rwlock;
pub mod semaphore;
pub mod condvar;
