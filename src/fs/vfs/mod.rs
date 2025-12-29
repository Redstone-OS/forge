//! Virtual File System

pub mod dentry;
pub mod file;
pub mod inode;
pub mod mount;
pub mod path;

pub use file::FileOps;
use file::{File, OpenFlags};
use inode::{DirEntry, FileMode, FileType, FsError, Inode, InodeNum, InodeOps};

/// Root VFS instance placeholder
pub struct RootVfs;
pub const ROOT_VFS: RootVfs = RootVfs;

impl RootVfs {
    /// Lock dummy (placeholder)
    pub fn lock(&self) -> &Self {
        self
    }

    /// Lookup placeholder
    pub fn lookup(&self, _path: &str) -> Result<Inode, FsError> {
        // Retorna NotFound para tudo por enquanto
        Err(FsError::NotFound)
    }
}
use crate::sync::Spinlock;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Árvore de inodes
static INODES: Spinlock<BTreeMap<InodeNum, Inode>> = Spinlock::new(BTreeMap::new());

/// Dummy Inode Ops para diretórios placeholder
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

/// Helper para criar inode de diretório
fn create_dir_inode(ino: InodeNum) -> Inode {
    Inode {
        ino,
        file_type: FileType::Directory,
        mode: FileMode(FileMode::OWNER_READ | FileMode::OWNER_EXEC), // Basic logic
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

/// Inicializa VFS e Hierarquia
pub fn init() {
    crate::kinfo!("(VFS) Inicializando...");

    let mut inodes = INODES.lock();

    // 1. Root /
    inodes.insert(0, create_dir_inode(0));

    // 2. Hierarquia solicitada
    // /system, /runtime, /state, /data, /users, /apps, /snapshots
    // IDs arbitrários para estrutura inicial
    let dirs = [
        (1, "system"),
        (2, "runtime"),
        (3, "state"),
        (4, "data"),
        (5, "users"),
        // (6, "apps"),
        // (7, "snapshots"),
        // (8, "dev"),  // Necessário para devfs
        // (9, "proc"), // Necessário para procfs
        // (10, "sys"), // Necessário para sysfs
        // (11, "tmp"), // Utils
    ];

    for (id, name) in dirs {
        inodes.insert(id, create_dir_inode(id));
        crate::kinfo!("(VFS) Criado diretório /", name);
    }
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
    if path == "/" {
        return Ok(0);
    }

    let mut current_ino: InodeNum = 0; // Raiz

    for component in path::PathComponents::new(path) {
        let inodes = INODES.lock();
        let inode = inodes.get(&current_ino).ok_or(FsError::NotFound)?;

        if let Some(next) = inode.ops.lookup(component) {
            current_ino = next;
        } else {
            // Fallback para os diretórios raiz estáticos que criamos no init
            // (Isso é um hack temporário pq o DummyOps não tem lookup real)
            // Em uma implementação real, o inode 0 teria um map de children.
            match (current_ino, component) {
                (0, "system") => current_ino = 1,
                (0, "runtime") => current_ino = 2,
                (0, "state") => current_ino = 3,
                (0, "data") => current_ino = 4,
                (0, "users") => current_ino = 5,
                (0, "apps") => current_ino = 6,
                (0, "snapshots") => current_ino = 7,
                (0, "dev") => current_ino = 8,
                (0, "proc") => current_ino = 9,
                (0, "sys") => current_ino = 10,
                (0, "tmp") => current_ino = 11,
                _ => return Err(FsError::NotFound),
            }
        }
    }

    Ok(current_ino)
}
