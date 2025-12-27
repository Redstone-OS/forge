//! # Syscall Flags
//!
//! Todas as flags usadas em syscalls.

/// Flags para operações de IO
pub mod io {
    pub const NONBLOCK: u32 = 1 << 0;
    pub const APPEND: u32 = 1 << 1;
    pub const SYNC: u32 = 1 << 2;
}

/// Flags para mapeamento de memória
pub mod map {
    pub const READ: u32 = 1 << 0;
    pub const WRITE: u32 = 1 << 1;
    pub const EXEC: u32 = 1 << 2;
    pub const SHARED: u32 = 1 << 3;
    pub const PRIVATE: u32 = 1 << 4;
    pub const FIXED: u32 = 1 << 5;
}

/// Flags para portas IPC
pub mod port {
    pub const SEND_ONLY: u32 = 1 << 0;
    pub const RECV_ONLY: u32 = 1 << 1;
    pub const BIDIRECTIONAL: u32 = 0;
}

/// Flags para mensagens IPC
pub mod msg {
    pub const NONBLOCK: u32 = 1 << 0;
    pub const URGENT: u32 = 1 << 1;
}

/// Flags para open
pub mod open {
    pub const RDONLY: u32 = 0;
    pub const WRONLY: u32 = 1;
    pub const RDWR: u32 = 2;
    pub const CREATE: u32 = 1 << 6;
    pub const EXCL: u32 = 1 << 7;
    pub const TRUNC: u32 = 1 << 9;
    pub const APPEND: u32 = 1 << 10;
    pub const DIRECTORY: u32 = 1 << 16;
}

/// Flags para alocação de memória
pub mod alloc {
    pub const ZEROED: u32 = 1 << 0;
    pub const GUARD: u32 = 1 << 1; // Página de guarda após alocação
}
