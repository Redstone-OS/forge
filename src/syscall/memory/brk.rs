//! # Heap Syscall (brk)
//!
//! sys_brk - gerenciamento de heap

use crate::syscall::SysResult;

/// sys_brk(new_brk) -> Result<current_brk>
///
/// Se new_brk == 0, retorna o brk atual.
/// Se new_brk > 0, tenta expandir/contrair o heap.
pub fn sys_brk(new_brk: usize) -> SysResult<usize> {
    // TODO: Obter task atual
    // TODO: Obter heap manager do address space
    // TODO: Expandir/contrair heap

    if new_brk == 0 {
        // Retornar brk atual
        // Por enquanto, retorna um valor placeholder
        Ok(0x1000_0000)
    } else {
        // Tentar alterar brk
        // Precisa verificar se há memória disponível
        let _ = new_brk;
        Err(crate::syscall::SysError::NotSupported)
    }
}
