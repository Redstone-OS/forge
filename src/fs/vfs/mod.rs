//! # Virtual File System (VFS)
//!
//! Camada de abstração que unifica todos os filesystems.
//!
//! ## Arquitetura
//!
//! ```text
//! Syscall → VFS → Path Resolution → Mount Table → Filesystem Backend
//! ```
//!
//! ## Hierarquia RedstoneOS
//!
//! | Diretório   | Tipo        | Descrição                         |
//! |-------------|-------------|-----------------------------------|
//! | /system     | Read-only   | SO imutável                       |
//! | /apps       | Read-write  | Aplicações instaladas             |
//! | /users      | Read-write  | Dados e config por usuário        |
//! | /devices    | Virtual     | Hardware abstraído                |
//! | /volumes    | Mount       | Partições lógicas                 |
//! | /runtime    | tmpfs       | Estado volátil                    |
//! | /state      | Persistente | Configurações do sistema          |
//! | /data       | Persistente | Dados globais                     |
//! | /net        | Virtual     | Rede como namespace               |
//! | /snapshots  | Read-only   | Histórico navegável               |
//! | /boot       | Read-only   | Boot mínimo                       |

pub mod dentry;
pub mod file;
pub mod inode;
pub mod mount;
pub mod path;

pub use file::FileOps;
use file::{File, OpenFlags};
use inode::{DirEntry, FileMode, FileType, FsError, Inode, InodeNum, InodeOps};

use crate::sync::Spinlock;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Instância raiz do VFS (placeholder)
pub struct RootVfs;
pub const ROOT_VFS: RootVfs = RootVfs;

impl RootVfs {
    /// Lock dummy (placeholder)
    pub fn lock(&self) -> &Self {
        self
    }

    /// Lookup placeholder
    pub fn lookup(&self, _path: &str) -> Result<Inode, FsError> {
        Err(FsError::NotFound)
    }
}

/// Árvore de inodes
static INODES: Spinlock<BTreeMap<InodeNum, Inode>> = Spinlock::new(BTreeMap::new());

/// Operações dummy para diretórios placeholder
struct DummyDirOps;

impl InodeOps for DummyDirOps {
    fn lookup(&self, _name: &str) -> Option<InodeNum> {
        None
    }
    fn read(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize, FsError> {
        Err(FsError::IsDirectory)
    }
    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize, FsError> {
        Err(FsError::IsDirectory)
    }
    fn readdir(&self) -> Result<Vec<DirEntry>, FsError> {
        Ok(Vec::new())
    }
}

static DUMMY_DIR_OPS: DummyDirOps = DummyDirOps;

/// Cria um inode de diretório
fn create_dir_inode(ino: InodeNum) -> Inode {
    Inode {
        ino,
        file_type: FileType::Directory,
        mode: FileMode(FileMode::OWNER_READ | FileMode::OWNER_EXEC),
        size: 0,
        nlink: 2,
        uid: 0,
        gid: 0,
        atime: 0,
        mtime: 0,
        ctime: 0,
        ops: &DUMMY_DIR_OPS,
    }
}

/// Inicializa o VFS e cria a hierarquia de diretórios
pub fn init() {
    crate::kinfo!("(VFS) Inicializando...");

    let mut inodes = INODES.lock();

    // Raiz /
    inodes.insert(0, create_dir_inode(0));

    // Hierarquia RedstoneOS
    let dirs = [
        (1, "system"),
        (2, "apps"),
        (3, "users"),
        (4, "devices"),
        (5, "volumes"),
        (6, "runtime"),
        (7, "state"),
        (8, "data"),
        (9, "net"),
        (10, "snapshots"),
        (11, "boot"),
    ];

    for (id, name) in dirs {
        inodes.insert(id, create_dir_inode(id));
        crate::kinfo!("(VFS) Criado /", name);
    }
}

/// Abre um arquivo
pub fn open(path: &str, flags: OpenFlags) -> Result<File, FsError> {
    let normalized = path::normalize(path);
    let ino = lookup(&normalized)?;

    let inodes = INODES.lock();
    let inode = inodes.get(&ino).ok_or(FsError::NotFound)?;

    Ok(File::new(inode as *const Inode, flags))
}

/// Resolve caminho para número de inode
fn lookup(path: &str) -> Result<InodeNum, FsError> {
    if path == "/" {
        return Ok(0);
    }

    let mut current_ino: InodeNum = 0;

    for component in path::PathComponents::new(path) {
        let inodes = INODES.lock();
        let inode = inodes.get(&current_ino).ok_or(FsError::NotFound)?;

        if let Some(next) = inode.ops.lookup(component) {
            current_ino = next;
        } else {
            // Fallback para diretórios raiz estáticos
            // TODO: Implementar lookup real via mount table
            match (current_ino, component) {
                (0, "system") => current_ino = 1,
                (0, "apps") => current_ino = 2,
                (0, "users") => current_ino = 3,
                (0, "devices") => current_ino = 4,
                (0, "volumes") => current_ino = 5,
                (0, "runtime") => current_ino = 6,
                (0, "state") => current_ino = 7,
                (0, "data") => current_ino = 8,
                (0, "net") => current_ino = 9,
                (0, "snapshots") => current_ino = 10,
                (0, "boot") => current_ino = 11,
                _ => return Err(FsError::NotFound),
            }
        }
    }

    Ok(current_ino)
}

/// Lê o conteúdo completo de um arquivo pelo caminho.
///
/// Esta função roteia para o backend correto:
/// - `/system/core/*` → InitRAMFS (bootstrap)
/// - `/system/services/*`, `/apps/*`, etc → FAT no disco
///
/// # Retorno
/// - `Some(bytes)` se o arquivo foi encontrado
/// - `None` se não encontrado
pub fn read_file(path: &str) -> Option<alloc::vec::Vec<u8>> {
    crate::ktrace!("(VFS) read_file():", path);

    // Rota 1: InitRAMFS para arquivos de bootstrap
    // O initramfs contém apenas /system/core/supervisor
    if path.starts_with("/system/core/") {
        crate::ktrace!("(VFS) Roteando para InitRAMFS");
        if let Some(data) = crate::fs::initramfs::lookup_file(path) {
            return Some(data.to_vec());
        }
    }

    // Rota 2: FAT no disco para todo o resto
    // Paths mapeados para FAT:
    // /system/services/* -> system/services/* no disco
    // /apps/* -> apps/* no disco
    crate::ktrace!("(VFS) Roteando para FAT");
    if let Some(data) = crate::fs::fat::read_file(path) {
        return Some(data);
    }

    // Fallback: tentar initramfs mesmo assim (para compatibilidade)
    if let Some(data) = crate::fs::initramfs::lookup_file(path) {
        return Some(data.to_vec());
    }

    None
}
