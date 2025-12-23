//! Tipos Primitivos do Sistema.
//!
//! Define aliases padrão para garantir consistência em todo o OS.

pub type Pid = usize; // Process ID
pub type Tid = usize; // Thread ID
pub type Uid = u32; // User ID
pub type Gid = u32; // Group ID
pub type Mode = u16; // File Mode/Permissions
pub type Dev = u64; // Device ID
pub type Ino = u64; // Inode Number
pub type Off = i64; // File Offset
pub type Time = i64; // Timestamp (Unix)

// File Descriptor
pub const STDIN_FILENO: usize = 0;
pub const STDOUT_FILENO: usize = 1;
pub const STDERR_FILENO: usize = 2;
