//! # Virtual File System (VFS) Layer
//!
//! O subsistema `fs` implementa a camada de abstração de arquivos do Redstone OS.
//! Ele fornece uma interface unificada (`VfsNode`, `VfsHandle`) para acessar diferentes
//! tipos de sistemas de arquivos (em memória, drivers, disco).
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Abstração (VFS):** Permite que o kernel trate arquivos, diretórios e dispositivos da mesma forma.
//! - **Initramfs:** Carrega o sistema de arquivos raiz inicial (TAR) da memória RAM.
//! - **DevFS:** Expõe dispositivos de kernel (como Serial / Console) como arquivos em `/dev`.
//!
//! ## 🏗️ Arquitetura dos Módulos
//!
//! | Módulo      | Responsabilidade | Estado Atual |
//! |-------------|------------------|--------------|
//! | `vfs`       | Define os Traits (`VfsNode`, `VfsHandle`) e o `Vfs` global. | **Síncrono:** Interface bloqueante básica. |
//! | `initramfs` | Parser de TAR (USTAR) Read-Only. | **Frágil:** Parser manual, sem checksum, assume UTF-8 válido. |
//! | `devfs`     | Filesystem sintético para `/dev`. | **Mínimo:** Suporta apenas `null` e `console`. |
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Simplicidade:** Interface limpa e fácil de implementar para novos FS.
//! - **Transparência:** O `Vfs::lookup` resolve caminhos de forma iterativa, fácil de debugar.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Lookup Linear O(N):** `vfs.rs` itera sobre listas de filhos para resolver caminhos. Em diretórios grandes, isso será lento.
//! - **Initramfs inseguro:** O parser TAR assume que os nomes de arquivos são UTF-8 válido (`unsafe { String::from_utf8_unchecked }`).
//!   - *Risco:* Um initramfs corrompido pode causar Undefined Behavior no kernel.
//! - **Falta de Cache:** Não existe *Page Cache* ou *Dentry Cache*. Cada leitura no `initramfs` copia bytes da RAM bruta.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Security)** Validar UTF-8 no parser do Initramfs ou usar `BString` (bytes puros).
//!   - *Impacto:* Evitar que nomes de arquivos malformados quebrem strings do Rust.
//! - [ ] **TODO: (Performance)** Implementar `DentryCache` no VFS global.
//!   - *Motivo:* Evitar parsing repetitivo de caminhos (ex: `/bin/init` não deve varrer `/`, depois `/bin` toda vez).
//! - [ ] **TODO: (Feature)** Suporte a **Mount Points**.
//!   - *Status:* Atualmente o VFS só tem um `root`. Precisamos montar `devfs` dentro de `initramfs/dev`.
//! - [ ] **TODO: (Concurrency)** Granularidade de Lock no VFS.
//!   - *Problema:* `ROOT_VFS` é um `Mutex` global. Todas as operações de arquivo do sistema bloqueiam umas às outras.

pub mod devfs;
pub mod initramfs;
pub mod test;
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
