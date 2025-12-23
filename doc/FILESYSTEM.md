# Sistema de Arquivos (VFS)

## 📋 Índice

- [Visão Geral](#visão-geral)
- [VFS (Virtual File System)](#vfs-virtual-file-system)
- [InitRAMFS](#initramfs)
- [DevFS](#devfs)

---

## Visão Geral

O Forge implementa um **Virtual File System (VFS)** que abstrai os detalhes de diferentes sistemas de arquivos, fornecendo uma interface unificada para o kernel e usuários. Todo acesso a arquivos passa pelo VFS.

### Estrutura do Módulo (`src/fs/`)
-   **`vfs.rs`**: Interfaces `Inode`, `File`, `FileSystem`.
-   **`initramfs.rs`**: Sistema de arquivos em RAM carregado no boot.
-   **`devfs.rs`**: Sistema de arquivos virtual para dispositivos (`/dev`).

---

## VFS (Virtual File System)

O VFS define traits que todos os sistemas de arquivos devem implementar.

### Traits Principais
```rust
pub trait FileSystem {
    fn root(&self) -> Arc<dyn Inode>;
    fn mount(&self, path: &str, fs: Arc<dyn FileSystem>) -> Result<()>;
}

pub trait Inode {
    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>>;
    fn open(&self) -> Result<Arc<dyn File>>;
    fn create(&self, name: &str, type_: FileType) -> Result<Arc<dyn Inode>>;
}
```

### Path Resolution
O VFS resolve caminhos como `/home/user/file.txt` atravessando Inodes recursivamente a partir da raiz (`/`).

---

## InitRAMFS

O **InitRAMFS** é um arquivo CPIO ou formato customizado carregado pelo bootloader junto com o kernel.
-   Contém programas essenciais (`init`, `sh`).
-   É montado como a raiz (`/`) inicialmente.
-   Permite que o kernel carregue drivers e módulos antes de montar o disco real.

---

## DevFS

O **DevFS** é um pseudo-filesystem montado tipicamente em `/dev`. Ele expõe drivers de dispositivo como arquivos.

-   `/dev/console`: Saída de texto.
-   `/dev/null`: Descarte de dados.
-   `/dev/serial`: Porta serial crua.

Isso permite que programas usem syscalls padrão (`read`, `write`) para interagir com hardware.
