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

pub use vfs::file::{File, FileOps};
pub use vfs::inode::{Inode, InodeOps};

// Re-export VFS (struct/trait if strictly needed, or just the module logic)
// Since VFS is a module here, this line might be redundant or wrong if VFS is not a struct.
// Checking previous fs/mod.rs error: 'no VFS in fs::vfs'.
// vfs/mod.rs defines functions like init, open. It doesn't define a VFS struct.
// But some code imports VFS. Maybe it means the module? 'pub use vfs as VFS'?
// Or maybe there IS a VFS struct missing?
// Given existing code tries to import it, I'll export what is available.
// If VFS struct is missing, I should create a dummy one or point to the module.
// For now, let's export Inode/File which are definitely missing.

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

pub mod rfs;

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
