//! # Standard Error Codes (Errno)
//!
//! Define os códigos de erro retornados pelo kernel.
//! Baseado no padrão POSIX para compatibilidade com ferramentas existentes.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Uniformidade:** Todas as syscalls retornam códigos padronizados.
//! - **Conversion:** Métodos `as_isize` facilitam o retorno negativo no registrador RAX.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Standard Compliance:** Seguir a numeração do Linux/POSIX evita reinvenção da roda e facilita porting de `std`.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Limited Scope:** Erros POSIX são focados em Arquivos e Processos. Eles descrevem mal erros de **IPC, Capabilities e Microkernel**.
//!   - *Problema:* O que retornar quando uma Capability é inválida? `EBADF`? `EPERM`? `EINVAL`? Nenhum serve perfeitamente.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Architecture)** Definir estratégia de erros para Capabilities.
//!   - *Opção A:* Mapear tudo para códigos POSIX (perde semântica).
//!   - *Opção B:* Estender `Errno` com códigos customizados (ex: `ECAP = 1000`).
//! - [ ] **TODO: (DevEx)** Implementar `impl Display` para imprimir mensagens de erro legíveis no log do kernel.
//!
//! --------------------------------------------------------------------------------
//!
//! Segue o padrão POSIX/Linux para facilitar compatibilidade futura e entendimento.
//! Valores negativos são usados em retornos de syscalls (isize).

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Errno {
    Success = 0,
    EPERM = 1,    // Operation not permitted
    ENOENT = 2,   // No such file or directory
    ESRCH = 3,    // No such process
    EINTR = 4,    // Interrupted system call
    EIO = 5,      // I/O error
    ENXIO = 6,    // No such device or address
    E2BIG = 7,    // Argument list too long
    ENOEXEC = 8,  // Exec format error
    EBADF = 9,    // Bad file number
    ECHILD = 10,  // No child processes
    EAGAIN = 11,  // Try again
    ENOMEM = 12,  // Out of memory
    EACCES = 13,  // Permission denied
    EFAULT = 14,  // Bad address
    EBUSY = 16,   // Device or resource busy
    EEXIST = 17,  // File exists
    EXDEV = 18,   // Cross-device link
    ENODEV = 19,  // No such device
    ENOTDIR = 20, // Not a directory
    EISDIR = 21,  // Is a directory
    EINVAL = 22,  // Invalid argument
    ENFILE = 23,  // File table overflow
    EMFILE = 24,  // Too many open files
    ENOTTY = 25,  // Not a typewriter
    EFBIG = 27,   // File too large
    ENOSPC = 28,  // No space left on device
    ESPIPE = 29,  // Illegal seek
    EROFS = 30,   // Read-only file system
    EMLINK = 31,  // Too many links
    EPIPE = 32,   // Broken pipe
    EDOM = 33,    // Math argument out of domain of func
    ERANGE = 34,  // Math result not representable
    ENOSYS = 38,  // Function not implemented

    // Redstone Specific
    ECAP = 1000, // Capability error (Invalid permissions/slot)
}

impl Errno {
    pub fn as_usize(self) -> usize {
        self as usize
    }

    pub fn as_isize(self) -> isize {
        -(self as i32) as isize
    }
}
