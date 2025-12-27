//! # Redstone OS Syscall Interface
//!
//! Fronteira definitiva entre Kernel (Ring 0) e Aplicações (Ring 3).
//!
//! ## Filosofia
//! - **Capability-First:** Syscalls operam sobre Handles, não recursos globais
//! - **Type-Safe:** Handles têm tipo, rights e refcounting
//! - **Table-Based:** Dispatch O(1) via lookup table
//!
//! ## Mecanismo
//! - **Instrução:** `syscall` (não int 0x80!)
//! - **Entry:** Ver arch/x86_64/syscall.rs
//!
//! ## Módulos
//!
//! | Módulo | Responsabilidade |
//! |--------|------------------|
//! | `abi` | Convenção de chamada, tipos compartilhados |
//! | `dispatch` | Dispatcher central (table-based) |
//! | `handle` | HandleTable, CapRights |
//! | `numbers` | Constantes de syscall (IMUTÁVEIS) |
//! | `error` | Códigos de erro |
//! | `fs` | open, close, read, write, stat |
//! | `process` | exit, spawn, wait, yield |
//! | `memory` | alloc, free, map, unmap |
//! | `ipc` | create_port, send, recv |
//! | `event` | poll |
//! | `time` | clock_get, sleep |
//! | `system` | sysinfo, debug |

// === Core ===
pub mod abi;
pub mod dispatch;
pub mod error;
pub mod handle;
pub mod numbers;

// === Implementações ===
pub mod event;
pub mod fs;
pub mod ipc;
pub mod memory;
pub mod process;
pub mod system;
pub mod time;

// === Testes ===
#[cfg(feature = "self_test")]
pub mod test;

// === Re-exports principais ===
pub use abi::{SyscallArgs, ABI_VERSION, REDSTONE_MAGIC};
pub use dispatch::syscall_dispatcher;
pub use error::{SysError, SysResult};
pub use handle::{Handle, HandleRights, HandleTable, HandleType};
pub use numbers::*;

/// Inicializa o subsistema de syscalls
pub fn init() {
    crate::kinfo!("(Syscall) Inicializando subsistema de syscalls...");

    // TODO: Setup MSRs para syscall instruction
    // init_syscall_msr();

    crate::kinfo!("(Syscall) ABI version=", abi::version::ABI_VERSION as u64);
    crate::kdebug!("(Syscall) Table size=", dispatch::table::TABLE_SIZE as u64);
}
