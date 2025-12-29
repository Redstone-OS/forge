//! # Syscall Interface
//!
//! Fronteira definitiva entre Kernel (Ring 0) e Userspace (Ring 3).
//!
//! ## Filosofia
//!
//! - **Capability-First**: Syscalls operam sobre Handles, não recursos globais
//! - **Type-Safe**: Handles têm tipo e rights verificados
//! - **Table-Based**: Dispatch O(1) via lookup table
//!
//! ## Convenção de Chamada
//!
//! ```text
//! Instrução: syscall (não int 0x80!)
//!
//! Entrada:
//!   RAX = número da syscall
//!   RDI = arg0
//!   RSI = arg1
//!   RDX = arg2
//!   R10 = arg3 (RCX é destruído por syscall)
//!   R8  = arg4
//!   R9  = arg5
//!
//! Saída:
//!   RAX = resultado (valor ou -errno)
//! ```
//!
//! ## Categorias
//!
//! | Categoria | Syscalls                         |
//! |-----------|----------------------------------|
//! | Process   | exit, spawn, wait, yield         |
//! | Memory    | alloc, free, map, unmap          |
//! | FS        | open, close, read, write, stat   |
//! | IPC       | port_create, send, recv          |
//! | Handle    | dup, close, transfer             |
//! | Time      | clock_get, sleep                 |
//! | System    | sysinfo, debug                   |
//! | Event     | poll                             |

// =============================================================================
// CORE
// =============================================================================

/// ABI e convenções
pub mod abi;

/// Dispatcher central
pub mod dispatch;

/// Códigos de erro
pub mod error;

/// Handle table e rights
pub mod handle;

/// Números de syscall
pub mod numbers;

// =============================================================================
// IMPLEMENTATIONS
// =============================================================================

/// Syscalls de evento (poll)
pub mod event;

/// Syscalls de filesystem
pub mod fs;

/// Syscalls de IPC
pub mod ipc;

/// Syscalls de memória
pub mod memory;

/// Syscalls de processo
pub mod process;

/// Syscalls de sistema
pub mod system;

/// Syscalls de tempo
pub mod time;

/// Syscalls de gráficos e input
pub mod gfx;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use abi::{SyscallArgs, ABI_VERSION};
pub use dispatch::syscall_dispatcher;
pub use error::{SysError, SysResult};
pub use handle::{Handle, HandleRights, HandleTable, HandleType};
pub use numbers::*;

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Inicializa o subsistema de syscalls
pub fn init() {
    crate::kinfo!("(Syscall) Inicializando interface de syscalls...");

    // Configurar MSRs para syscall/sysret
    unsafe {
        crate::arch::x86_64::syscall::init();
    }

    crate::kinfo!("(Syscall) ABI version:", abi::ABI_VERSION as u64);
    crate::kinfo!("(Syscall) Interface inicializada");
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
