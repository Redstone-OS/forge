/// Arquivo: sys/syscall.rs
///
/// Propósito: Definição dos Números de Chamada de Sistema (Syscall Numbers).
/// Deve estar sincronizado com o user space (libc/syscall wrappers).
/// TODO: Porque ta aqui?

//! Números de Syscall

// Processo
pub const SYS_EXIT: usize = 60;
pub const SYS_FORK: usize = 57;
pub const SYS_EXECVE: usize = 59;
pub const SYS_WAIT4: usize = 61;
pub const SYS_GETPID: usize = 39;
pub const SYS_KILL: usize = 62;

// Arquivos
pub const SYS_READ: usize = 0;
pub const SYS_WRITE: usize = 1;
pub const SYS_OPEN: usize = 2;
pub const SYS_CLOSE: usize = 3;
pub const SYS_STAT: usize = 4;
pub const SYS_FSTAT: usize = 5;
pub const SYS_LSEEK: usize = 8;
pub const SYS_IOCTL: usize = 16;

// Memória
pub const SYS_MMAP: usize = 9;
pub const SYS_MPROTECT: usize = 10;
pub const SYS_MUNMAP: usize = 11;
pub const SYS_BRK: usize = 12;

// Outros
pub const SYS_GETCWD: usize = 79;
pub const SYS_CHDIR: usize = 80;
pub const SYS_RENAME: usize = 82;
pub const SYS_MKDIR: usize = 83;
pub const SYS_RMDIR: usize = 84;
pub const SYS_LINK: usize = 86;
pub const SYS_UNLINK: usize = 87;

// Redstone específico
pub const SYS_DEBUG_PRINT: usize = 1000;
pub const SYS_THREAD_SPAWN: usize = 1001;
pub const SYS_CHANNEL_CREATE: usize = 1002;

// IPC - Shared Memory
pub const SYS_SHM_CREATE: usize = 200;
pub const SYS_SHM_MAP: usize = 201;
pub const SYS_SHM_UNMAP: usize = 202;

// IPC - Message Passing
pub const SYS_MSG_SEND: usize = 210;
pub const SYS_MSG_RECV: usize = 211;

// IPC - Ports
pub const SYS_PORT_CREATE: usize = 220;
pub const SYS_PORT_CONNECT: usize = 221;
