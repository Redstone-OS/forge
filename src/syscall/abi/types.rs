//! # Syscall Types
//!
//! Estruturas de dados compartilhadas entre kernel e userspace.

/// Vetor de IO (similar a iovec)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IoVec {
    pub base: *mut u8,
    pub len: usize,
}

impl IoVec {
    pub const fn empty() -> Self {
        Self {
            base: core::ptr::null_mut(),
            len: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.base.is_null() && self.len > 0
    }

    /// Verifica se está em espaço de usuário (não kernel)
    pub fn is_user_space(&self) -> bool {
        let addr = self.base as usize;
        // User space em x86_64 canonical: < 0x0000_8000_0000_0000
        addr < 0x0000_8000_0000_0000
    }
}

/// Estrutura de tempo (nanosegundos)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TimeSpec {
    pub seconds: u64,
    pub nanoseconds: u32,
    pub _pad: u32,
}

impl TimeSpec {
    pub const fn zero() -> Self {
        Self {
            seconds: 0,
            nanoseconds: 0,
            _pad: 0,
        }
    }

    pub fn from_millis(ms: u64) -> Self {
        Self {
            seconds: ms / 1000,
            nanoseconds: ((ms % 1000) * 1_000_000) as u32,
            _pad: 0,
        }
    }

    pub fn to_millis(&self) -> u64 {
        self.seconds * 1000 + (self.nanoseconds / 1_000_000) as u64
    }
}

/// Informações de arquivo (stat)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Stat {
    /// Tipo de arquivo
    pub mode: u32,
    /// Tamanho em bytes
    pub size: u64,
    /// Tempo de criação
    pub ctime: TimeSpec,
    /// Tempo de modificação
    pub mtime: TimeSpec,
    /// Tempo de acesso
    pub atime: TimeSpec,
    /// Número de hard links
    pub nlink: u32,
    /// ID do dispositivo
    pub dev: u32,
    /// Inode
    pub ino: u64,
}

/// Tipos de arquivo (mode)
pub mod file_type {
    pub const REGULAR: u32 = 0o100000;
    pub const DIRECTORY: u32 = 0o040000;
    pub const SYMLINK: u32 = 0o120000;
    pub const CHARDEV: u32 = 0o020000;
    pub const BLOCKDEV: u32 = 0o060000;
    pub const FIFO: u32 = 0o010000;
    pub const SOCKET: u32 = 0o140000;
}

/// Entrada de diretório
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DirEntry {
    /// Inode
    pub ino: u64,
    /// Tipo de arquivo
    pub file_type: u8,
    /// Tamanho do nome
    pub name_len: u8,
    /// Nome (null-terminated, max 255)
    pub name: [u8; 256],
}

impl DirEntry {
    pub const fn empty() -> Self {
        Self {
            ino: 0,
            file_type: 0,
            name_len: 0,
            name: [0u8; 256],
        }
    }

    pub fn name_str(&self) -> &str {
        let len = self.name_len as usize;
        core::str::from_utf8(&self.name[..len]).unwrap_or("")
    }
}

/// Descritor de poll
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PollFd {
    /// Handle a monitorar
    pub handle: u32,
    /// Eventos de interesse
    pub events: u16,
    /// Eventos retornados
    pub revents: u16,
}

/// Eventos de poll
pub mod poll_events {
    pub const IN: u16 = 1 << 0; // Dados disponíveis para leitura
    pub const OUT: u16 = 1 << 1; // Espaço disponível para escrita
    pub const ERR: u16 = 1 << 2; // Erro
    pub const HUP: u16 = 1 << 3; // Hangup
    pub const NVAL: u16 = 1 << 4; // Handle inválido
}

/// Tipos de clock
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockId {
    Realtime = 0,
    Monotonic = 1,
    ProcessCpu = 2,
    ThreadCpu = 3,
}

/// Whence para lseek
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekWhence {
    Set = 0,     // Início do arquivo
    Current = 1, // Posição atual
    End = 2,     // Final do arquivo
}
