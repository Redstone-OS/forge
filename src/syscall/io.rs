//! Syscalls de IO
//!
//! Leitura e escrita vetorizada via handles.
//! Todo IO passa por handles - não há file descriptors.

use super::abi::IoVec;
use super::error::{SysError, SysResult};
use core::slice;

/// Escrita vetorizada em um handle.
///
/// # Syscall
/// `SYS_WRITEV (0x41)` - Args: (handle, iov_ptr, iov_cnt, flags)
///
/// # Argumentos
/// - `handle`: Handle com direito WRITE
/// - `iov_ptr`: Ponteiro para array de IoVec
/// - `iov_cnt`: Número de IoVecs
/// - `flags`: io_flags (NONBLOCK, SYNC, etc)
///
/// # Retorno
/// Total de bytes escritos ou erro
pub fn sys_writev(
    handle: usize,
    iov_ptr: usize,
    iov_cnt: usize,
    _flags: usize,
) -> SysResult<usize> {
    // Validação básica
    if iov_ptr == 0 || iov_cnt == 0 {
        return Err(SysError::InvalidArgument);
    }

    // Limite de segurança
    if iov_cnt > 16 {
        return Err(SysError::InvalidArgument);
    }

    // HACK TEMPORÁRIO: Handle 0 = console (debug)
    // No futuro, cada processo receberá handles de console via IPC
    if handle == 0 {
        return write_console(iov_ptr, iov_cnt);
    }

    // TODO: Para outros handles:
    // 1. Obter HandleEntry do processo atual
    // 2. Verificar direito WRITE
    // 3. Despachar para o driver/subsistema correto

    crate::kwarn!("(Syscall) sys_writev: Handle {} não implementado", handle);
    Err(SysError::BadHandle)
}

/// Leitura vetorizada de um handle.
///
/// # Syscall
/// `SYS_READV (0x40)` - Args: (handle, iov_ptr, iov_cnt, flags)
///
/// # Argumentos
/// - `handle`: Handle com direito READ
/// - `iov_ptr`: Ponteiro para array de IoVec (buffers de destino)
/// - `iov_cnt`: Número de IoVecs
/// - `flags`: io_flags (NONBLOCK, etc)
///
/// # Retorno
/// Total de bytes lidos ou erro
pub fn sys_readv(handle: usize, iov_ptr: usize, iov_cnt: usize, _flags: usize) -> SysResult<usize> {
    if iov_ptr == 0 || iov_cnt == 0 {
        return Err(SysError::InvalidArgument);
    }

    if iov_cnt > 16 {
        return Err(SysError::InvalidArgument);
    }

    // HACK TEMPORÁRIO: Handle 0 = console (stdin de teclado)
    if handle == 0 {
        // TODO: Ler do buffer de teclado
        crate::kwarn!("(Syscall) sys_readv: Leitura de console não implementada");
        return Err(SysError::NotImplemented);
    }

    // TODO: Para outros handles, dispatch por tipo

    crate::kwarn!("(Syscall) sys_readv: Handle {} não implementado", handle);
    Err(SysError::BadHandle)
}

/// Helper: Escreve no console (debug)
fn write_console(iov_ptr: usize, iov_cnt: usize) -> SysResult<usize> {
    // TODO: Validar que ponteiros pertencem ao userspace do processo atual
    // Por enquanto, confiamos (INSEGURO em produção!)

    let iovecs = unsafe { slice::from_raw_parts(iov_ptr as *const IoVec, iov_cnt) };

    let mut total = 0usize;

    for iov in iovecs {
        if iov.base.is_null() || iov.len == 0 {
            continue;
        }

        // Ler dados do buffer
        let data = unsafe { slice::from_raw_parts(iov.base as *const u8, iov.len) };

        // Tentar imprimir como UTF-8
        match core::str::from_utf8(data) {
            Ok(s) => crate::kprint!("{}", s),
            Err(_) => {
                // Fallback: imprimir bytes como chars
                for &b in data {
                    if b.is_ascii_graphic() || b == b' ' || b == b'\n' {
                        crate::kprint!("{}", b as char);
                    } else {
                        crate::kprint!(".");
                    }
                }
            }
        }

        total += iov.len;
    }

    Ok(total)
}
