//! # Memory Mapping Syscalls (POSIX-like)
//!
//! Implementação de mmap/munmap no estilo POSIX para compatibilidade.
//!
//! ## Flags de Proteção
//!
//! | Flag | Valor | Descrição |
//! |------|-------|-----------|
//! | `PROT_NONE` | 0 | Sem acesso |
//! | `PROT_READ` | 1 | Leitura permitida |
//! | `PROT_WRITE` | 2 | Escrita permitida |
//! | `PROT_EXEC` | 4 | Execução permitida |
//!
//! ## Flags de Mapeamento
//!
//! | Flag | Valor | Descrição |
//! |------|-------|-----------|
//! | `MAP_SHARED` | 0x01 | Modificações visíveis a outros processos |
//! | `MAP_PRIVATE` | 0x02 | Copy-on-write |
//! | `MAP_FIXED` | 0x10 | Usar endereço exato |
//! | `MAP_ANONYMOUS` | 0x20 | Não associado a arquivo |
//!
//! ## Status
//!
//! - MAP_ANONYMOUS: Stub (TODO: implementar)
//! - File-backed: Não suportado

use crate::rmm::virt::aspace::vma::{MemoryIntent, Protection, VmaFlags};
use crate::syscall::error::{SysError, SysResult};

// =============================================================================
// CONSTANTES DE PROTEÇÃO
// =============================================================================

/// Sem acesso
pub const PROT_NONE: u32 = 0;

/// Leitura permitida
pub const PROT_READ: u32 = 1;

/// Escrita permitida
pub const PROT_WRITE: u32 = 2;

/// Execução permitida
pub const PROT_EXEC: u32 = 4;

// =============================================================================
// CONSTANTES DE FLAGS
// =============================================================================

/// Modificações são compartilhadas
pub const MAP_SHARED: u32 = 0x01;

/// Modificações são privadas (COW)
pub const MAP_PRIVATE: u32 = 0x02;

/// Usar endereço exato fornecido
pub const MAP_FIXED: u32 = 0x10;

/// Mapeamento anônimo (não associado a arquivo)
pub const MAP_ANONYMOUS: u32 = 0x20;

// =============================================================================
// BASE DE MAPEAMENTO
// =============================================================================

/// Base para mapeamentos mmap (512 MB)
const MMAP_BASE: u64 = 0x2000_0000;

/// Limite de mapeamentos mmap (1 GB)
const MMAP_LIMIT: u64 = 0x4000_0000;

// =============================================================================
// SYSCALLS
// =============================================================================

/// sys_mmap - Mapeia memória no espaço do processo
///
/// # Argumentos
///
/// * `hint` - Endereço sugerido (0 = kernel escolhe)
/// * `size` - Tamanho em bytes
/// * `prot` - Proteções (PROT_*)
/// * `flags` - Flags de mapeamento (MAP_*)
/// * `fd` - File descriptor (-1 para anônimo)
/// * `offset` - Offset no arquivo (ignorado para anônimo)
///
/// # Retorna
///
/// * `Ok(addr)` - Endereço onde foi mapeado
/// * `Err` - Erro de mapeamento
pub fn sys_mmap(
    hint: usize,
    size: usize,
    prot: u32,
    flags: u32,
    fd: i32,
    offset: u64,
) -> SysResult<usize> {
    // Validar tamanho
    if size == 0 || size > 0x7FFF_FFFF_0000 {
        return Err(SysError::InvalidArgument);
    }

    // File-backed mapping não suportado ainda
    if flags & MAP_ANONYMOUS == 0 && fd >= 0 {
        crate::kwarn!("(Syscall) mmap: file-backed not supported");
        return Err(SysError::NotSupported);
    }

    // MAP_ANONYMOUS: memória zerada
    if flags & MAP_ANONYMOUS != 0 {
        // TODO: Implementar mapeamento anônimo real
        // - Escolher endereço (hint ou calcular)
        // - Alocar frames zerados
        // - Mapear no address space
        // - Registrar VMA
        crate::kwarn!("(Syscall) mmap: anonymous mapping TODO");
        let _ = (hint, prot, offset);
        return Err(SysError::NotSupported);
    }

    Err(SysError::NotSupported)
}

/// sys_munmap - Remove mapeamento de memória
///
/// # Argumentos
///
/// * `addr` - Endereço base do mapeamento
/// * `size` - Tamanho em bytes
///
/// # Retorna
///
/// * `Ok(0)` - Mapeamento removido
/// * `Err` - Erro
pub fn sys_munmap(addr: usize, size: usize) -> SysResult<usize> {
    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    // TODO: Implementar unmapping
    // - Lookup VMA
    // - Unmap páginas
    // - Liberar frames
    // - Remover VMA
    let _ = addr;
    crate::kwarn!("(Syscall) munmap: not implemented");
    Err(SysError::NotSupported)
}

/// sys_mprotect - Altera proteções de uma região
///
/// # Argumentos
///
/// * `addr` - Endereço base (alinhado a página)
/// * `size` - Tamanho em bytes
/// * `prot` - Novas proteções (PROT_*)
pub fn sys_mprotect(addr: usize, size: usize, prot: u32) -> SysResult<usize> {
    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    // TODO: Implementar mprotect
    // - Validar alinhamento
    // - Lookup VMA
    // - Atualizar proteções na page table
    // - Flush TLB
    let _ = (addr, prot);
    crate::kwarn!("(Syscall) mprotect: not implemented");
    Err(SysError::NotSupported)
}

// =============================================================================
// FUNÇÕES DE CONVERSÃO
// =============================================================================

/// Converte flags PROT_* para Protection do RMM
#[allow(unused)]
fn convert_prot(prot: u32) -> Protection {
    if prot & (PROT_WRITE | PROT_EXEC) == (PROT_WRITE | PROT_EXEC) {
        Protection::RWX
    } else if prot & PROT_WRITE != 0 {
        Protection::RW
    } else if prot & PROT_EXEC != 0 {
        Protection::RX
    } else if prot & PROT_READ != 0 {
        Protection::RO
    } else {
        Protection::NONE
    }
}

/// Converte flags MAP_* para VmaFlags do RMM
#[allow(unused)]
fn convert_flags(flags: u32) -> VmaFlags {
    let mut f = VmaFlags::empty();
    if flags & MAP_SHARED != 0 {
        f = f.union(VmaFlags::SHARED);
    }
    if flags & MAP_PRIVATE != 0 {
        f = f.union(VmaFlags::COW);
    }
    f
}

/// Infere o MemoryIntent baseado nos flags
#[allow(unused)]
fn infer_intent(prot: u32, flags: u32) -> MemoryIntent {
    if flags & MAP_ANONYMOUS != 0 {
        if prot & PROT_EXEC != 0 {
            MemoryIntent::Code
        } else {
            MemoryIntent::AnonMmap
        }
    } else {
        MemoryIntent::FileMmap
    }
}
