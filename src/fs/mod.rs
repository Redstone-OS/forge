//! Sistema de Arquivos Virtual (VFS).
//!
//! Submódulos:
//! - `vfs`: Traits e estrutura central.
//! - `initramfs`: Parser TAR para o disco inicial.
//! - `devfs`: Dispositivos virtuais (/dev).

pub mod devfs;
pub mod initramfs;
pub mod vfs;

use alloc::sync::Arc;

/// Inicializa o subsistema de arquivos.
pub fn init(boot_info: &'static crate::core::handoff::BootInfo) {
    crate::kinfo!("[Init] FS: Initializing VFS...");

    // 1. Procurar Initramfs no BootInfo
    if boot_info.initramfs_addr != 0 && boot_info.initramfs_size > 0 {
        crate::kinfo!(
            "[Init] FS: Found Initramfs at {:#x} ({} bytes)",
            boot_info.initramfs_addr,
            boot_info.initramfs_size
        );

        // Criar slice unsafe para a memória do initramfs
        let data = unsafe {
            core::slice::from_raw_parts(
                boot_info.initramfs_addr as *const u8,
                boot_info.initramfs_size as usize,
            )
        };

        // Montar Initramfs como raiz
        let initfs = Arc::new(initramfs::Initramfs::new(data));
        vfs::ROOT_VFS.lock().mount_root(initfs);

        crate::kinfo!("[Init] FS: Root filesystem mounted.");
    } else {
        crate::kwarn!("[Init] FS: No Initramfs found! System will halt shortly.");
    }
}
