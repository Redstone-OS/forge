# Guia de Implementação: `fs/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE.

---

## 1. PROPÓSITO

Virtual File System - abstração unificada de armazenamento.

---

## 2. ESTRUTURA

```
fs/
├── mod.rs              ✅ JÁ EXISTE
├── vfs/
│   ├── mod.rs
│   ├── dentry.rs       → Cache de diretórios
│   ├── file.rs         → Arquivo aberto
│   ├── inode.rs        → Metadados
│   ├── mount.rs        → Pontos de montagem
│   └── path.rs         → Parsing de caminhos
├── devfs/
│   └── mod.rs          → /dev
├── initramfs/
│   └── mod.rs          → Boot filesystem
├── procfs/
│   ├── mod.rs
│   └── proc.rs         → /proc
├── sysfs/
│   ├── mod.rs
│   └── sys.rs          → /sys
└── tmpfs/
    ├── mod.rs
    └── tmpfs.rs        → RAM disk
```

---

## 3. REGRAS

### ❌ NUNCA:
- Acessar disco diretamente (use drivers/block)
- Seguir symlinks infinitamente
- Permitir path traversal fora de mount

### ✅ SEMPRE:
- Validar caminhos
- Usar handles para arquivos abertos
- Implementar FileOps trait

---

## 4. IMPLEMENTAÇÕES

### 4.1 `vfs/inode.rs`

```rust
//! Inode - metadados de arquivo

/// Tipo de arquivo
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    CharDevice,
    BlockDevice,
    Fifo,
    Socket,
}

/// Permissões
#[derive(Debug, Clone, Copy)]
pub struct FileMode(pub u32);

impl FileMode {
    pub const OWNER_READ: u32 = 0o400;
    pub const OWNER_WRITE: u32 = 0o200;
    pub const OWNER_EXEC: u32 = 0o100;
    pub const GROUP_READ: u32 = 0o040;
    pub const GROUP_WRITE: u32 = 0o020;
    pub const GROUP_EXEC: u32 = 0o010;
    pub const OTHER_READ: u32 = 0o004;
    pub const OTHER_WRITE: u32 = 0o002;
    pub const OTHER_EXEC: u32 = 0o001;
    
    pub fn can_read(&self, is_owner: bool) -> bool {
        if is_owner {
            (self.0 & Self::OWNER_READ) != 0
        } else {
            (self.0 & Self::OTHER_READ) != 0
        }
    }
}

/// Número de inode
pub type InodeNum = u64;

/// Inode
pub struct Inode {
    /// Número único
    pub ino: InodeNum,
    /// Tipo de arquivo
    pub file_type: FileType,
    /// Permissões
    pub mode: FileMode,
    /// Tamanho em bytes
    pub size: u64,
    /// Links count
    pub nlink: u32,
    /// UID do dono
    pub uid: u32,
    /// GID do grupo
    pub gid: u32,
    /// Timestamps
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    /// Operações específicas
    pub ops: &'static dyn InodeOps,
}

/// Operações de inode
pub trait InodeOps: Send + Sync {
    /// Lookup em diretório
    fn lookup(&self, name: &str) -> Option<InodeNum>;
    
    /// Ler dados
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, FsError>;
    
    /// Escrever dados
    fn write(&self, offset: u64, buf: &[u8]) -> Result<usize, FsError>;
    
    /// Listar diretório
    fn readdir(&self) -> Result<alloc::vec::Vec<DirEntry>, FsError>;
}

/// Entrada de diretório
pub struct DirEntry {
    pub name: alloc::string::String,
    pub ino: InodeNum,
    pub file_type: FileType,
}

/// Erro de filesystem
#[derive(Debug)]
pub enum FsError {
    NotFound,
    NotDirectory,
    IsDirectory,
    PermissionDenied,
    IoError,
    ReadOnly,
    NoSpace,
}
```

### 4.2 `vfs/file.rs`

```rust
//! Arquivo aberto

use super::inode::{Inode, FsError};
use crate::sync::Mutex;

/// Flags de abertura
#[derive(Debug, Clone, Copy)]
pub struct OpenFlags(pub u32);

impl OpenFlags {
    pub const READ: u32 = 1;
    pub const WRITE: u32 = 2;
    pub const APPEND: u32 = 4;
    pub const CREATE: u32 = 8;
    pub const TRUNCATE: u32 = 16;
}

/// Arquivo aberto
pub struct File {
    /// Inode associado
    inode: *const Inode,
    /// Posição atual
    offset: Mutex<u64>,
    /// Flags de abertura
    flags: OpenFlags,
}

impl File {
    /// Cria arquivo aberto
    pub fn new(inode: *const Inode, flags: OpenFlags) -> Self {
        Self {
            inode,
            offset: Mutex::new(0),
            flags,
        }
    }
    
    /// Lê dados
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, FsError> {
        let inode = unsafe { &*self.inode };
        let mut offset = self.offset.lock();
        let bytes = inode.ops.read(*offset, buf)?;
        *offset += bytes as u64;
        Ok(bytes)
    }
    
    /// Escreve dados
    pub fn write(&self, buf: &[u8]) -> Result<usize, FsError> {
        let inode = unsafe { &*self.inode };
        let mut offset = self.offset.lock();
        let bytes = inode.ops.write(*offset, buf)?;
        *offset += bytes as u64;
        Ok(bytes)
    }
    
    /// Seek
    pub fn seek(&self, position: u64) {
        *self.offset.lock() = position;
    }
}
```

### 4.3 `vfs/path.rs`

```rust
//! Parsing de caminhos

/// Iterador sobre componentes de caminho
pub struct PathComponents<'a> {
    remaining: &'a str,
}

impl<'a> PathComponents<'a> {
    pub fn new(path: &'a str) -> Self {
        // Remover / inicial
        let path = path.strip_prefix('/').unwrap_or(path);
        Self { remaining: path }
    }
}

impl<'a> Iterator for PathComponents<'a> {
    type Item = &'a str;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        
        match self.remaining.find('/') {
            Some(pos) => {
                let component = &self.remaining[..pos];
                self.remaining = &self.remaining[pos + 1..];
                Some(component)
            }
            None => {
                let component = self.remaining;
                self.remaining = "";
                Some(component)
            }
        }
    }
}

/// Verifica se caminho é absoluto
pub fn is_absolute(path: &str) -> bool {
    path.starts_with('/')
}

/// Normaliza caminho (remove . e ..)
pub fn normalize(path: &str) -> alloc::string::String {
    use alloc::vec::Vec;
    use alloc::string::String;
    
    let mut components: Vec<&str> = Vec::new();
    
    for comp in PathComponents::new(path) {
        match comp {
            "" | "." => continue,
            ".." => { components.pop(); }
            _ => components.push(comp),
        }
    }
    
    let mut result = String::from("/");
    for (i, comp) in components.iter().enumerate() {
        if i > 0 {
            result.push('/');
        }
        result.push_str(comp);
    }
    
    result
}
```

### 4.4 `vfs/mod.rs`

```rust
//! Virtual File System

pub mod dentry;
pub mod file;
pub mod inode;
pub mod mount;
pub mod path;

use inode::{Inode, InodeNum, FsError};
use file::{File, OpenFlags};
use crate::sync::Spinlock;
use alloc::collections::BTreeMap;

/// Árvore de inodes
static INODES: Spinlock<BTreeMap<InodeNum, Inode>> = 
    Spinlock::new(BTreeMap::new());

/// Inicializa VFS
pub fn init() {
    crate::kinfo!("(VFS) Inicializando...");
    // Criar inode raiz
}

/// Abre arquivo
pub fn open(path: &str, flags: OpenFlags) -> Result<File, FsError> {
    let normalized = path::normalize(path);
    
    // Resolver caminho
    let ino = lookup(&normalized)?;
    
    // Pegar inode
    let inodes = INODES.lock();
    let inode = inodes.get(&ino).ok_or(FsError::NotFound)?;
    
    Ok(File::new(inode as *const Inode, flags))
}

/// Resolve caminho para inode
fn lookup(path: &str) -> Result<InodeNum, FsError> {
    let mut current_ino: InodeNum = 0; // Raiz
    
    for component in path::PathComponents::new(path) {
        let inodes = INODES.lock();
        let inode = inodes.get(&current_ino).ok_or(FsError::NotFound)?;
        
        if let Some(next) = inode.ops.lookup(component) {
            current_ino = next;
        } else {
            return Err(FsError::NotFound);
        }
    }
    
    Ok(current_ino)
}
```

### 4.5 `initramfs/mod.rs`

```rust
//! InitramFS - filesystem em memória do boot

use crate::mm::VirtAddr;
use crate::fs::vfs::inode::{Inode, InodeOps, FileType, FsError, DirEntry};

/// Inode do initramfs
struct InitramfsInode {
    data: *const u8,
    size: usize,
}

impl InodeOps for InitramfsInode {
    fn lookup(&self, _name: &str) -> Option<u64> {
        // TODO: implementar lookup em CPIO/TAR
        None
    }
    
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, FsError> {
        let offset = offset as usize;
        if offset >= self.size {
            return Ok(0);
        }
        
        let to_read = buf.len().min(self.size - offset);
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.data.add(offset),
                buf.as_mut_ptr(),
                to_read
            );
        }
        Ok(to_read)
    }
    
    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize, FsError> {
        Err(FsError::ReadOnly)
    }
    
    fn readdir(&self) -> Result<alloc::vec::Vec<DirEntry>, FsError> {
        // TODO: parse CPIO entries
        Ok(alloc::vec::Vec::new())
    }
}

/// Carrega initramfs da memória
pub fn init(addr: VirtAddr, size: usize) {
    crate::kinfo!("(InitramFS) Carregando de addr=", addr.as_u64());
    crate::kinfo!("(InitramFS) Tamanho:", size as u64);
    // TODO: parse e montar
}
```

---

## 5. ORDEM DE IMPLEMENTAÇÃO

1. `vfs/path.rs` - Parsing de caminhos
2. `vfs/inode.rs` - Estrutura de inode
3. `vfs/file.rs` - Arquivo aberto
4. `vfs/mod.rs` - Core VFS
5. `initramfs/mod.rs` - Boot FS
6. `devfs/mod.rs` - /dev
7. `tmpfs/tmpfs.rs` - RAM disk

---

## 6. DEPENDÊNCIAS

Pode importar de:
- `crate::sync`
- `crate::mm`
- `crate::klib`
- `crate::drivers::block` (para FS reais)

---

## 7. CHECKLIST

- [ ] PathComponents itera corretamente
- [ ] Inodes são imutáveis (exceto via operações)
- [ ] File tracking position
- [ ] Initramfs é read-only
