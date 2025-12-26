//! # Memory Management Syscalls
//!
//! A interface primária para controle do espaço de endereçamento virtual (VMM) do processo.
//!
//! ## 🎯 Propósito
//! - **Allocation:** Pedir mais memória ao kernel (`sbrk` morreu, vida longa ao `mmap`).
//! - **Sharing:** Mapear objetos (arquivos, memória compartilhada) no espaço de endereço.
//!
//! ## 🏗️ Arquitetura
//! - **Page Granularity:** Todas as operações são arredondadas para 4KiB (Page Size).
//! - **VMA (Virtual Memory Area):** O kernel mantém uma lista de regiões válidas. Acessar fora delas gera Page Fault (SIGSEGV).
//!
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **No ASLR:** Atualmente o `sys_alloc` é determinístico. Falta randomização de endereço base (ASLR) para segurança contra exploits.
//! - **No Overcommit:** O kernel promete memória que talvez não tenha? Precisamos definir a política de *Overcommit*.
//!
//! ## 🛠️ TODOs
//! - [ ] **TODO: (Feature)** Implementar **Shared Memory** real (mapear o mesmo frame físico em dois processos).
//! - [ ] **TODO: (Security)** Implementar **ASLR** (Address Space Layout Randomization).
//! - [ ] **TODO: (Reliability)** Implementar **Guard Pages** (páginas não mapeadas entre alocações para pegar buffer overflows lineares).
//!
//! --------------------------------------------------------------------------------
//!
//! Alocação e mapeamento de memória virtual.

use super::abi::map_flags;
use super::error::{SysError, SysResult};

/// Aloca memória virtual.
///
/// # Syscall
/// `SYS_ALLOC (0x10)` - Args: (size, flags)
///
/// # Argumentos
/// - `size`: Tamanho em bytes (será arredondado para páginas)
/// - `flags`: map_flags (READ, WRITE, EXEC)
///
/// # Retorno
/// Endereço da região alocada
pub fn sys_alloc(size: usize, flags: usize) -> SysResult<usize> {
    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    // Arredondar para páginas
    let page_size = 4096usize;
    let pages = (size + page_size - 1) / page_size;

    let _flags = flags as u32;

    // TODO: Usar VMM para alocar páginas no espaço do processo
    // TODO: Mapear com permissões corretas

    crate::kwarn!("(Syscall) sys_alloc: não implementado size=", size as u64);
    Err(SysError::NotImplemented)
}

/// Libera memória alocada.
///
/// # Syscall
/// `SYS_FREE (0x11)` - Args: (addr, size)
///
/// # Argumentos
/// - `addr`: Endereço retornado por SYS_ALLOC
/// - `size`: Tamanho original
pub fn sys_free(addr: usize, size: usize) -> SysResult<usize> {
    if addr == 0 {
        return Err(SysError::BadAddress);
    }

    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    // Verificar alinhamento
    if addr % 4096 != 0 {
        return Err(SysError::BadAlignment);
    }

    // TODO: Usar VMM para desmapear e liberar páginas

    crate::kwarn!("(Syscall) sys_free: não implementado addr=", addr as u64);
    Err(SysError::NotImplemented)
}

/// Mapeia região de memória ou handle.
///
/// # Syscall
/// `SYS_MAP (0x12)` - Args: (addr, size, flags, handle)
///
/// # Argumentos
/// - `addr`: Endereço desejado (0 = kernel escolhe)
/// - `size`: Tamanho do mapeamento
/// - `flags`: map_flags
/// - `handle`: Handle de memória/arquivo (0 = anônimo)
///
/// # Retorno
/// Endereço do mapeamento
pub fn sys_map(addr: usize, size: usize, flags: usize, handle: usize) -> SysResult<usize> {
    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    // Verificar alinhamento se endereço fixo
    let flags_u32 = flags as u32;
    if flags_u32 & map_flags::FIXED != 0 && addr % 4096 != 0 {
        return Err(SysError::BadAlignment);
    }

    // TODO: Implementar mapeamento
    // - Se handle == 0: mapeamento anônimo
    // - Se handle != 0: verificar tipo (Memory/File) e mapear

    crate::kwarn!("(Syscall) sys_map: não implementado addr=", addr as u64);
    crate::klog!(" size=", size as u64, " handle=", handle as u64);
    crate::knl!();
    Err(SysError::NotImplemented)
}

/// Remove mapeamento de memória.
///
/// # Syscall
/// `SYS_UNMAP (0x13)` - Args: (addr, size)
pub fn sys_unmap(addr: usize, size: usize) -> SysResult<usize> {
    if addr == 0 || size == 0 {
        return Err(SysError::InvalidArgument);
    }

    if addr % 4096 != 0 {
        return Err(SysError::BadAlignment);
    }

    // TODO: Usar VMM para remover mapeamento

    crate::kwarn!("(Syscall) sys_unmap: não implementado addr=", addr as u64);
    Err(SysError::NotImplemented)
}
