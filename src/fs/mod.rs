//! # File System (FS)
//!
//! Abstração unificada de armazenamento.
//!
//! ## Arquitetura VFS
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                   SYSCALL LAYER                     │
//! │         open() read() write() close() stat()        │
//! └─────────────────────────────────────────────────────┘
//!                          ↓
//! ┌─────────────────────────────────────────────────────┐
//! │                       VFS                           │
//! │   Path Resolution → Dentry Cache → Inode → File     │
//! └─────────────────────────────────────────────────────┘
//!                          ↓
//! ┌─────────────────────────────────────────────────────┐
//! │              FILESYSTEM BACKENDS                    │
//! │   InitramFS │ DevFS │ ProcFS │ SysFS │ TmpFS        │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Filesystems
//!
//! | FS        | Tipo       | Persistente | Propósito              |
//! |-----------|------------|-------------|------------------------|
//! | initramfs | Read-only  | Não         | Boot inicial           |
//! | devfs     | Virtual    | Não         | /dev/null, /dev/sda    |
//! | procfs    | Virtual    | Não         | /proc (estado kernel)  |
//! | sysfs     | Virtual    | Não         | /sys (dispositivos)    |
//! | tmpfs     | RAM-backed | Não         | Storage temporário     |

// =============================================================================
// VIRTUAL FILE SYSTEM
// =============================================================================

/// Core VFS (path resolution, file ops)
pub mod vfs;

pub use vfs::{File, FileOps, Inode, SuperBlock, VFS};

// =============================================================================
// FILESYSTEM IMPLEMENTATIONS
// =============================================================================

/// DevFS (/dev)
pub mod devfs;

/// InitramFS (boot)
pub mod initramfs;

/// ProcFS (/proc)
pub mod procfs;

/// SysFS (/sys)
pub mod sysfs;

/// TmpFS (volatile storage)
pub mod tmpfs;

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Inicializa o VFS e monta filesystems virtuais
pub fn init() {
    crate::kinfo!("(FS) Inicializando VFS...");
    vfs::init();

    crate::kinfo!("(FS) Montando filesystems virtuais...");
    // devfs::mount("/dev");
    // procfs::mount("/proc");
    // sysfs::mount("/sys");
    // tmpfs::mount("/tmp");

    crate::kinfo!("(FS) Filesystem inicializado");
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
