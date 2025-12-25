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
    crate::kinfo!("Inicializando VFS...");

    // 1. Procurar Initramfs no BootInfo
    if boot_info.initramfs_addr != 0 && boot_info.initramfs_size > 0 {
        crate::kinfo!(
            "Encontrado \"initfs\" em {:#x} ({} bytes)",
            boot_info.initramfs_addr,
            boot_info.initramfs_size
        );

        // Criar slice unsafe para a memória do initramfs
        crate::kinfo!("VFS: criando slice...");
        let data = unsafe {
            core::slice::from_raw_parts(
                boot_info.initramfs_addr as *const u8,
                boot_info.initramfs_size as usize,
            )
        };
        crate::kinfo!("VFS: slice OK, len={}", data.len());

        // Montar Initramfs como raiz
        crate::kinfo!("VFS: parsing initramfs...");
        let initfs = Arc::new(initramfs::Initramfs::new(data));
        crate::kinfo!("VFS: initfs OK, montando raiz...");
        vfs::ROOT_VFS.lock().mount_root(initfs);

        crate::kinfo!("Sistema de arquivos raiz montado.");
    } else {
        crate::kwarn!("Initramfs não encontrado! Sistema irá parar em breve.");
    }
}
