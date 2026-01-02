//! # File System (FS)
//!
//! Abstração unificada de armazenamento para o RedstoneOS.
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
//! │       InitramFS │ FAT │ RFS (futuro)                │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Hierarquia RedstoneOS
//!
//! ```text
//! /
//! ├─ system/     # SO imutável (services, drivers, manifests)
//! ├─ apps/       # Aplicações instaladas pelo usuário
//! ├─ users/      # Dados e config por usuário
//! ├─ devices/    # Hardware abstraído
//! ├─ volumes/    # Partições lógicas
//! ├─ runtime/    # Estado volátil (tmpfs)
//! ├─ state/      # Estado persistente pequeno
//! ├─ data/       # Dados globais
//! ├─ net/        # Rede como namespace
//! ├─ snapshots/  # Histórico navegável
//! └─ boot/       # Boot mínimo
//! ```

// =============================================================================
// VIRTUAL FILE SYSTEM
// =============================================================================

/// Core VFS (path resolution, file ops)
pub mod vfs;

pub use vfs::file::{File, FileOps};
pub use vfs::inode::{Inode, InodeOps};

// =============================================================================
// FILESYSTEM IMPLEMENTATIONS
// =============================================================================

/// InitramFS (boot) - TAR-based initial ramdisk
pub mod initramfs;

/// FAT Filesystem (FAT16/FAT32)
pub mod fat;

/// RFS - Redstone File System (futuro)
pub mod rfs;

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Inicializa o VFS e monta filesystems
pub fn init() {
    crate::kinfo!("(FS) Inicializando VFS...");
    vfs::init();

    crate::kinfo!("(FS) Inicializando módulo FAT...");
    fat::init();

    crate::kinfo!("(FS) Filesystem inicializado");
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
