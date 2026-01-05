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

    // Espera por um dispositivo de storage antes de inicializar filesystems
    // O sistema precisa de disco para funcionar - não adianta continuar sem
    wait_for_storage();

    crate::kinfo!("(FS) Inicializando módulo FAT...");
    fat::init();

    crate::kinfo!("(FS) Filesystem inicializado");
}

/// Aguarda até que um dispositivo de storage esteja disponível.
///
/// O sistema operacional não pode funcionar sem acesso a disco,
/// então bloqueamos aqui até encontrar pelo menos um dispositivo.
///
/// **Timeout**: 5 segundos (50 tentativas x 100ms)
fn wait_for_storage() {
    const MAX_RETRIES: u32 = 50;
    const RETRY_DELAY_LOOPS: u32 = 100_000;

    crate::kinfo!("(FS) Aguardando dispositivo de storage...");

    for retry in 0..MAX_RETRIES {
        // Verifica se há algum dispositivo de bloco
        if crate::drivers::storage::get_first_device().is_some() {
            crate::kinfo!("(FS) Dispositivo de storage encontrado!");
            return;
        }

        // Log a cada 10 tentativas
        if retry > 0 && retry % 10 == 0 {
            crate::kinfo!(
                "(FS) Aguardando storage... tentativa",
                retry,
                "/",
                MAX_RETRIES
            );
        }

        // Busy wait simples
        for _ in 0..RETRY_DELAY_LOOPS {
            core::hint::spin_loop();
        }
    }

    // Timeout atingido
    crate::kerror!("(FS) CRÍTICO: Nenhum dispositivo de storage após timeout!");
    crate::kerror!("(FS) O sistema continuará apenas com InitRAMFS.");
    // Não panic - deixa o sistema tentar boot mínimo com InitRAMFS
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(feature = "self_test")]
pub mod test;
