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
    crate::kinfo!("(VFS) Inicializando subsistema de arquivos...");

    // 1. Procurar Initramfs no BootInfo
    if boot_info.initramfs_addr != 0 && boot_info.initramfs_size > 0 {
        crate::kdebug!(
            "(VFS) Encontrado disco inicial 'initfs' em {:#x} ({} bytes)",
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
        crate::kinfo!("(VFS) Montando Initramfs...");
        let initfs = Arc::new(initramfs::Initramfs::new(data));
        vfs::ROOT_VFS.lock().mount_root(initfs);

        crate::kinfo!("(VFS) Sistema de arquivos raiz montado com sucesso");
    } else {
        crate::kwarn!(
            "(VFS) ATENÇÃO: Initramfs não encontrado! O sistema não poderá carregar o /init"
        );
    }
}
