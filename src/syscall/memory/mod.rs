//! # Memory Syscalls
//!
//! Syscalls para alocação e gerenciamento de memória virtual.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                             Memory Syscalls                             │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
//! │   │  sys_alloc  │  │  sys_free   │  │  sys_map    │  │ sys_unmap   │    │
//! │   │    0x10     │  │    0x11     │  │    0x12     │  │    0x13     │    │
//! │   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘    │
//! │          │                │                │                │           │
//! │          └────────────────┴────────────────┴────────────────┘           │
//! │                                   │                                     │
//! │                                   ▼                                     │
//! │                          ┌─────────────────┐                            │
//! │                          │   RMM (phys +   │                            │
//! │                          │   AddressSpace) │                            │
//! │                          └─────────────────┘                            │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Syscalls
//!
//! ### Core (0x10-0x14)
//!
//! | Número | Nome          | Descrição             |
//! |--------|---------------|-----------------------|
//! | 0x10   | `SYS_ALLOC`   | Aloca memória virtual |
//! | 0x11   | `SYS_FREE`    | Libera memória        |
//! | 0x12   | `SYS_MAP`     | Mapeia região (mmap)  |
//! | 0x13   | `SYS_UNMAP`   | Remove mapeamento     |
//! | 0x14   | `SYS_MPROTECT`| Altera proteções      |
//!
//! ### Extended (0x15-0x1D)
//!
//! | Número | Nome                | Descrição              |
//! |--------|---------------------|------------------------|
//! | 0x15   | `SYS_MEMINFO`       | Info de memória        |
//! | 0x16   | `SYS_ALLOC_AT`      | Aloca em endereço fixo |
//! | 0x17   | `SYS_SHM_CREATE`    | Cria região SHM        |
//! | 0x18   | `SYS_SHM_ATTACH`    | Mapeia região SHM      |
//! | 0x19   | `SYS_SHM_RELEASE`   | Libera região SHM      |
//! | 0x1A   | `SYS_CLOSE_MAPPING` | Fecha mapeamento       |
//! | 0x1B   | `SYS_MSYNC`         | Sincroniza mapeamento  |
//! | 0x1C   | `SYS_MADVISE`       | Dicas de uso           |
//! | 0x1D   | `SYS_SHM_GET_SIZE`  | Tamanho de SHM         |
//!
//! ## Módulos
//!
//! - [`alloc`] - sys_alloc, sys_free, sys_map, sys_unmap, sys_mprotect
//! - [`mmap`] - sys_mmap, sys_munmap (POSIX-like)
//! - [`brk`] - sys_brk (heap break POSIX)
//! - [`madvise`] - sys_madvise (dicas de uso)
//! - [`vmo`] - Virtual Memory Objects (futuro)

pub mod alloc;
pub mod brk;
pub mod madvise;
pub mod mmap;
pub mod vmo;

// Re-exports públicos
pub use alloc::*;
pub use brk::sys_brk;
pub use madvise::{
    sys_madvise, MADV_DONTNEED, MADV_FREE, MADV_NORMAL, MADV_RANDOM, MADV_SEQUENTIAL, MADV_WILLNEED,
};
pub use mmap::{
    sys_mmap, sys_munmap, MAP_ANONYMOUS, MAP_FIXED, MAP_PRIVATE, MAP_SHARED, PROT_EXEC, PROT_NONE,
    PROT_READ, PROT_WRITE,
};

// Constantes de flags exportadas
pub use alloc::{ALLOC_COMMIT, ALLOC_GUARD, ALLOC_ZEROED};
