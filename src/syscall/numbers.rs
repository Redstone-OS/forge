//! Números das System Calls (ABI).
//!
//! Segue parcialmente a convenção Linux x86_64 para facilitar ferramentas,
//! mas pode divergir onde necessário para o Redstone.

// File Operations
pub const SYS_READ: usize = 0;
pub const SYS_WRITE: usize = 1;
pub const SYS_OPEN: usize = 2;
pub const SYS_CLOSE: usize = 3;

// Process Operations
pub const SYS_SCHED_YIELD: usize = 24;
pub const SYS_GETPID: usize = 39;
pub const SYS_FORK: usize = 57;
pub const SYS_EXECVE: usize = 59;
pub const SYS_EXIT: usize = 60;
pub const SYS_WAIT4: usize = 61;
pub const SYS_KILL: usize = 62;

// Memory Operations
pub const SYS_MMAP: usize = 9;
pub const SYS_MUNMAP: usize = 11;

// Erros (errno-style, negativos)
pub const EPERM: isize = -1; // Operation not permitted
pub const ENOENT: isize = -2; // No such file or directory
pub const EIO: isize = -5; // I/O error
pub const EBADF: isize = -9; // Bad file number
pub const ENOMEM: isize = -12; // Out of memory
pub const EFAULT: isize = -14; // Bad address
pub const EINVAL: isize = -22; // Invalid argument
pub const ENOSYS: isize = -38; // Function not implemented
