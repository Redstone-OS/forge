//! # Filesystem Types
//!
//! Tipos compartilhados entre kernel e userspace para operações de filesystem.

use alloc::string::String;

// =============================================================================
// OPEN FLAGS
// =============================================================================

/// Flags para sys_open
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct OpenFlags(pub u32);

impl OpenFlags {
    /// Somente leitura
    pub const O_RDONLY: u32 = 0;
    /// Somente escrita
    pub const O_WRONLY: u32 = 1;
    /// Leitura e escrita
    pub const O_RDWR: u32 = 2;
    /// Máscara de acesso
    pub const O_ACCMODE: u32 = 3;

    /// Criar se não existir
    pub const O_CREATE: u32 = 0x0100;
    /// Truncar arquivo existente
    pub const O_TRUNC: u32 = 0x0200;
    /// Append mode
    pub const O_APPEND: u32 = 0x0400;
    /// Falhar se existir (com O_CREATE)
    pub const O_EXCL: u32 = 0x0800;
    /// Abrir diretório
    pub const O_DIRECTORY: u32 = 0x1000;

    pub fn can_read(&self) -> bool {
        (self.0 & Self::O_ACCMODE) != Self::O_WRONLY
    }

    pub fn can_write(&self) -> bool {
        (self.0 & Self::O_ACCMODE) != Self::O_RDONLY
    }

    pub fn is_create(&self) -> bool {
        (self.0 & Self::O_CREATE) != 0
    }

    pub fn is_directory(&self) -> bool {
        (self.0 & Self::O_DIRECTORY) != 0
    }
}

// =============================================================================
// SEEK WHENCE
// =============================================================================

/// Whence para sys_seek
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum SeekWhence {
    /// Do início do arquivo
    Set = 0,
    /// Da posição atual
    Cur = 1,
    /// Do fim do arquivo
    End = 2,
}

impl SeekWhence {
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            0 => Some(Self::Set),
            1 => Some(Self::Cur),
            2 => Some(Self::End),
            _ => None,
        }
    }
}

// =============================================================================
// FILE STAT
// =============================================================================

/// Tipo de arquivo
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum FileType {
    Unknown = 0,
    Regular = 1,
    Directory = 2,
    Symlink = 3,
    CharDevice = 4,
    BlockDevice = 5,
    Fifo = 6,
    Socket = 7,
}

/// Informações de arquivo (retornado por stat/fstat)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileStat {
    /// Tipo de arquivo
    pub file_type: u8,
    /// Permissões (mode)
    pub mode: u16,
    /// Padding
    pub _pad: u8,
    /// Tamanho em bytes
    pub size: u64,
    /// Número de hard links
    pub nlink: u32,
    /// UID do dono
    pub uid: u32,
    /// GID do grupo
    pub gid: u32,
    /// Padding
    pub _pad2: u32,
    /// Tempo de último acesso (ms desde epoch)
    pub atime: u64,
    /// Tempo de última modificação (ms desde epoch)
    pub mtime: u64,
    /// Tempo de criação (ms desde epoch)
    pub ctime: u64,
}

impl FileStat {
    pub const SIZE: usize = core::mem::size_of::<Self>();

    pub fn zeroed() -> Self {
        Self {
            file_type: 0,
            mode: 0,
            _pad: 0,
            size: 0,
            nlink: 0,
            uid: 0,
            gid: 0,
            _pad2: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        }
    }
}

// =============================================================================
// DIRECTORY ENTRY
// =============================================================================

/// Entrada de diretório (retornado por getdents)
///
/// Layout em memória:
/// | offset | size | field       |
/// |--------|------|-------------|
/// | 0      | 8    | ino         |
/// | 8      | 2    | rec_len     |
/// | 10     | 1    | file_type   |
/// | 11     | 1    | name_len    |
/// | 12     | N    | name[N]     |
#[derive(Debug)]
#[repr(C, packed)]
pub struct DirEntryHeader {
    /// Número do inode (pode ser 0 se não suportado)
    pub ino: u64,
    /// Tamanho total desta entrada (incluindo padding)
    pub rec_len: u16,
    /// Tipo de arquivo
    pub file_type: u8,
    /// Tamanho do nome (sem null terminator)
    pub name_len: u8,
    // name bytes seguem imediatamente
}

impl DirEntryHeader {
    pub const HEADER_SIZE: usize = 12;

    /// Calcula tamanho alinhado para uma entrada
    pub fn calc_rec_len(name_len: usize) -> usize {
        // Header + name + padding para alinhar em 8 bytes
        let total = Self::HEADER_SIZE + name_len;
        (total + 7) & !7
    }
}

/// Helper para construir DirEntry no buffer
pub struct DirEntryBuilder;

impl DirEntryBuilder {
    /// Escreve uma entrada de diretório no buffer
    /// Retorna quantos bytes foram escritos, ou None se não couber
    pub fn write(buf: &mut [u8], ino: u64, file_type: FileType, name: &str) -> Option<usize> {
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len().min(255);
        let rec_len = DirEntryHeader::calc_rec_len(name_len);

        if buf.len() < rec_len {
            return None;
        }

        // Header
        buf[0..8].copy_from_slice(&ino.to_le_bytes());
        buf[8..10].copy_from_slice(&(rec_len as u16).to_le_bytes());
        buf[10] = file_type as u8;
        buf[11] = name_len as u8;

        // Name
        buf[12..12 + name_len].copy_from_slice(&name_bytes[..name_len]);

        // Zerar padding
        for b in &mut buf[12 + name_len..rec_len] {
            *b = 0;
        }

        Some(rec_len)
    }
}

// =============================================================================
// FILESYSTEM STAT
// =============================================================================

/// Informações do filesystem (retornado por statfs)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FsStat {
    /// Tipo de filesystem (magic number)
    pub fs_type: u32,
    /// Tamanho do bloco
    pub block_size: u32,
    /// Total de blocos
    pub total_blocks: u64,
    /// Blocos livres
    pub free_blocks: u64,
    /// Total de inodes
    pub total_inodes: u64,
    /// Inodes livres
    pub free_inodes: u64,
    /// Tamanho máximo de nome
    pub max_name_len: u32,
    /// Padding
    pub _pad: u32,
}

// =============================================================================
// PATH UTILITIES
// =============================================================================

/// Extrai string de path do userspace
pub fn path_from_user(ptr: usize, len: usize) -> Result<String, crate::syscall::error::SysError> {
    use crate::syscall::error::SysError;

    if len == 0 || len > 4096 {
        return Err(SysError::InvalidArgument);
    }

    // Validar ponteiro
    if ptr == 0 {
        return Err(SysError::BadAddress);
    }

    // TODO: Proper copy_from_user with page table validation
    // Por agora, assumimos que o ponteiro é válido
    let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };

    // Converter para String
    match core::str::from_utf8(slice) {
        Ok(s) => Ok(String::from(s)),
        Err(_) => Err(SysError::InvalidArgument),
    }
}
