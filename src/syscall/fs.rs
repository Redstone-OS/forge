//! Implementação de Syscalls de Arquivo/IO.

use super::numbers::*;
use core::slice;
use core::str;

/// Escreve em um descritor de arquivo.
///
/// # Arguments
/// * `fd` - File Descriptor (1=stdout, 2=stderr)
/// * `buf_ptr` - Ponteiro para o buffer de dados (Userspace)
/// * `len` - Tamanho do buffer
pub fn sys_write(fd: usize, buf_ptr: usize, len: usize) -> isize {
    // 1. Validação básica de descritores
    if fd != 1 && fd != 2 {
        // Por enquanto só suportamos stdout/stderr
        return EBADF;
    }

    // 2. Validação de Ponteiro (Segurança Crítica)
    if buf_ptr == 0 {
        return EFAULT;
    }
    // TODO: Verificar se o range [buf_ptr, buf_ptr+len] pertence ao userspace.
    // Atualmente estamos rodando em Ring 0 flat, então confiamos (perigo!).

    // 3. Acesso à memória
    let data = unsafe { slice::from_raw_parts(buf_ptr as *const u8, len) };

    // 4. Converter e Imprimir
    // Tenta imprimir como UTF-8, se falhar, imprime bytes brutos
    match str::from_utf8(data) {
        Ok(s) => {
            crate::kprint!("{}", s);
        }
        Err(_) => {
            for &b in data {
                crate::kprint!("{}", b as char);
            }
        }
    }

    len as isize
}

/// Lê de um descritor de arquivo.
pub fn sys_read(fd: usize, _buf_ptr: usize, _len: usize) -> isize {
    if fd > 2 {
        return EBADF;
    }
    // TODO: Implementar leitura de teclado (stdin)
    // Requer buffer de teclado implementado na Fase 6
    crate::kwarn!("sys_read not implemented yet");
    -1 // Error
}
