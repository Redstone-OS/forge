//! # Filesystem Syscalls
//!
//! Implementação de todas as syscalls de filesystem do RedstoneOS.
//!
//! ## Organização
//!
//! | Range     | Módulo      | Descrição                    |
//! |-----------|-------------|------------------------------|
//! | 0x60-0x67 | `io`        | Operações básicas de I/O     |
//! | 0x68-0x6B | `meta`      | Metadados e permissões       |
//! | 0x6C-0x6F | `dir`       | Operações de diretório       |
//! | 0x70-0x73 | `file`      | Manipulação de arquivos      |
//! | 0x74-0x76 | `link`      | Links simbólicos             |
//! | 0x77-0x7A | `mount`     | Operações de montagem        |
//! | 0x7B-0x7F | `ctrl`      | Controle avançado            |

pub mod ctrl;
pub mod dir;
pub mod file;
pub mod handle;
pub mod io;
pub mod link;
pub mod meta;
pub mod mount;
pub mod types;

// Re-export all wrappers
pub use ctrl::*;
pub use dir::*;
pub use file::*;
pub use io::*;
pub use link::*;
pub use meta::*;
pub use mount::*;
