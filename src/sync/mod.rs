//! Primitivas de Sincronização.
//!
//! Abstração sobre spinlocks para garantir thread-safety no kernel.
//! Futuramente pode incluir Mutexes que desabilitam interrupções (IrqSafeMutex).

// Re-exporta o Mutex da crate `spin` por enquanto.
// Isso facilita mudar a implementação no futuro sem alterar o código consumidor.
pub use spin::{Mutex, MutexGuard};

/// Wrapper para garantir inicialização preguiçosa segura.
pub use spin::Lazy;
