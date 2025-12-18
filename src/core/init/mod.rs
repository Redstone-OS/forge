//! Sistema de Inicialização por Fases
//!
//! Gerencia a ordem de inicialização dos subsistemas do kernel.
//!
//! # Ordem de Inicialização
//! 1. CPU (GDT, IDT, interrupções)
//! 2. Memória (PMM, VMM)
//! 3. Scheduler
//! 4. Processos/Threads
//! 5. IPC
//! 6. VFS
//! 7. Drivers
//! 8. Segurança
//! 9. Rede
//! 10. Userspace init
//!
//! # TODOs
//! - TODO(prioridade=alta, versão=v1.0): Implementar sistema de fases
//! - TODO(prioridade=alta, versão=v1.0): Criar macro init_subsystem!

/// Fases de inicialização do kernel
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum InitPhase {
    /// Fase 1: CPU (GDT, IDT, interrupções)
    Cpu = 1,
    /// Fase 2: Memória (PMM, VMM)
    Memory = 2,
    /// Fase 3: Scheduler
    Scheduler = 3,
    /// Fase 4: Processos/Threads
    Process = 4,
    /// Fase 5: IPC
    Ipc = 5,
    /// Fase 6: VFS
    Vfs = 6,
    /// Fase 7: Drivers
    Drivers = 7,
    /// Fase 8: Segurança
    Security = 8,
    /// Fase 9: Rede
    Network = 9,
    /// Fase 10: Userspace init
    Userspace = 10,
}

/// Macro para registrar função de init
///
/// # Exemplo
/// ```rust
/// #[init_subsystem(InitPhase::Memory)]
/// fn mm_init() {
///     // Inicializa memória
/// }
/// ```
///
/// # TODOs
/// - TODO(prioridade=alta, versão=v1.0): Implementar macro
#[macro_export]
macro_rules! init_subsystem {
    ($phase:expr, $func:ident) => {
        // TODO: Implementar registro de init
    };
}
