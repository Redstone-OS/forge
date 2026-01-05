//! # Heap Syscall (brk)
//!
//! Gerenciamento do heap break no estilo POSIX.
//!
//! ## Comportamento
//!
//! - `sys_brk(0)` → Retorna o brk atual
//! - `sys_brk(new_brk)` → Tenta mover o brk para new_brk
//!
//! ## TODO
//!
//! Implementar expansão/contração real do heap com mapeamento de páginas.

use crate::syscall::error::{SysError, SysResult};

/// Base do heap (256 MB)
const HEAP_BASE: u64 = 0x1000_0000;

/// sys_brk(new_brk) -> Result<current_brk>
///
/// Se new_brk == 0, retorna o brk atual.
/// Se new_brk > 0, tenta expandir/contrair o heap.
///
/// # Argumentos
///
/// * `new_brk` - Novo endereço do break (0 = query)
///
/// # Retorna
///
/// * `Ok(brk)` - Endereço atual do break
/// * `Err(OutOfMemory)` - Não foi possível expandir
pub fn sys_brk(new_brk: usize) -> SysResult<usize> {
    // Obter task atual
    let current_brk = {
        let current = crate::sched::core::CURRENT.lock();
        if let Some(task) = current.as_ref() {
            task.heap_next
        } else {
            HEAP_BASE
        }
    };

    if new_brk == 0 {
        // Query: retorna brk atual
        return Ok(current_brk as usize);
    }

    let new_brk = new_brk as u64;

    if new_brk < HEAP_BASE {
        // Não pode ir abaixo do base
        return Err(SysError::InvalidArgument);
    }

    if new_brk < current_brk {
        // TODO: Contrair heap (liberar páginas)
        // Por enquanto: apenas atualiza o ponteiro
        let mut current = crate::sched::core::CURRENT.lock();
        if let Some(task) = current.as_mut() {
            task.heap_next = new_brk;
        }
        return Ok(new_brk as usize);
    }

    if new_brk > current_brk {
        // TODO: Expandir heap (mapear páginas)
        // Por enquanto: apenas atualiza o ponteiro sem mapear
        // Páginas serão mapeadas on-demand via page fault
        let mut current = crate::sched::core::CURRENT.lock();
        if let Some(task) = current.as_mut() {
            task.heap_next = new_brk;
        }
        return Ok(new_brk as usize);
    }

    Ok(current_brk as usize)
}
