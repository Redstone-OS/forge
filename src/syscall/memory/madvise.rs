//! # Memory Advise Syscall
//!
//! Fornece dicas ao kernel sobre o padrão de uso de uma região de memória.
//!
//! ## Dicas Suportadas
//!
//! | Constante | Valor | Descrição |
//! |-----------|-------|-----------|
//! | `MADV_NORMAL` | 0 | Uso normal (padrão) |
//! | `MADV_RANDOM` | 1 | Acesso aleatório esperado |
//! | `MADV_SEQUENTIAL` | 2 | Acesso sequencial esperado |
//! | `MADV_WILLNEED` | 3 | Páginas serão acessadas em breve |
//! | `MADV_DONTNEED` | 4 | Páginas não serão mais necessárias |
//! | `MADV_FREE` | 8 | Liberar páginas imediatamente |
//!
//! ## Status
//!
//! Atualmente implementado como noop para maioria das dicas.
//! Dicas informativas (NORMAL, RANDOM, SEQUENTIAL) não requerem ação.

use crate::syscall::error::{SysError, SysResult};

/// Uso normal (padrão)
pub const MADV_NORMAL: i32 = 0;

/// Espera acesso aleatório
pub const MADV_RANDOM: i32 = 1;

/// Espera acesso sequencial
pub const MADV_SEQUENTIAL: i32 = 2;

/// Páginas serão necessárias em breve
pub const MADV_WILLNEED: i32 = 3;

/// Páginas não são mais necessárias
pub const MADV_DONTNEED: i32 = 4;

/// Liberar páginas imediatamente
pub const MADV_FREE: i32 = 8;

/// sys_madvise(addr, size, advice) -> Result<0>
///
/// Fornece dicas de uso de memória ao kernel.
///
/// # Argumentos
///
/// * `addr` - Endereço base da região (alinhado a página)
/// * `size` - Tamanho da região em bytes
/// * `advice` - Dica de uso (MADV_*)
///
/// # Retorna
///
/// * `Ok(0)` - Dica aceita (pode não ter efeito)
/// * `Err(InvalidArgument)` - Dica desconhecida ou size = 0
pub fn sys_madvise(addr: usize, size: usize, advice: i32) -> SysResult<usize> {
    if size == 0 {
        return Err(SysError::InvalidArgument);
    }

    match advice {
        MADV_NORMAL | MADV_RANDOM | MADV_SEQUENTIAL => {
            // Dicas informativas, não requerem ação
            // TODO: Ajustar readahead do page cache
        }
        MADV_WILLNEED => {
            // TODO: Prefetch das páginas
            // Carregar páginas na memória antes de serem acessadas
            crate::ktrace!("(Syscall) madvise: WILLNEED at", addr as u64);
        }
        MADV_DONTNEED => {
            // TODO: Marcar páginas como descartáveis
            // Podem ser reclamadas sem swap
            crate::ktrace!("(Syscall) madvise: DONTNEED at", addr as u64);
        }
        MADV_FREE => {
            // TODO: Liberar páginas imediatamente
            // Similar a DONTNEED mas mais agressivo
            crate::ktrace!("(Syscall) madvise: FREE at", addr as u64);
        }
        _ => {
            crate::kwarn!("(Syscall) madvise: unknown advice=", advice as u64);
            return Err(SysError::InvalidArgument);
        }
    }

    let _ = addr;
    Ok(0)
}
