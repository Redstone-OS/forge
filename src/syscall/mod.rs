//! Syscall Handler
//!
//! Implementa syscalls básicos para processos userspace.

#![allow(dead_code)]

use crate::drivers::legacy::serial;

/// Syscall numbers
pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT: u64 = 60;

/// File descriptors
pub const STDOUT: u64 = 1;
pub const STDERR: u64 = 2;

// ============================================================================
// TODO(prioridade=ALTA, versão=v1.0): MIGRAR PARA SYSCALL/SYSRET
// ============================================================================
//
// ⚠️ ATENÇÃO: Usando int 0x80 ao invés de syscall instruction!
//
// PROBLEMA: int 0x80 é lento (legacy) e não é o método moderno.
//
// SOLUÇÃO ATUAL: int 0x80 para simplicidade
//
// RISCOS:
// - Performance ruim comparado a syscall/sysret
// - Não é o padrão x86_64 moderno
//
// SOLUÇÕES FUTURAS:
// 1. Implementar syscall/sysret instruction
// 2. Configurar MSRs (STAR, LSTAR, SFMASK)
// 3. Muito mais rápido (~10x)
// ============================================================================

/// Handler de syscalls via int 0x80
///
/// # Arguments
///
/// * `num` - Número do syscall (RAX)
/// * `arg1` - Primeiro argumento (RDI)
/// * `arg2` - Segundo argumento (RSI)
/// * `arg3` - Terceiro argumento (RDX)
///
/// # Returns
///
/// Valor de retorno (RAX)
pub fn syscall_handler(num: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    match num {
        SYS_WRITE => syscall_write(arg1, arg2, arg3),
        SYS_EXIT => syscall_exit(arg1),
        _ => {
            serial::println("[SYSCALL] Syscall desconhecido");
            u64::MAX // -1 (erro)
        }
    }
}

/// SYS_WRITE - Escrever em file descriptor
///
/// # Arguments
///
/// * `fd` - File descriptor (1 = stdout, 2 = stderr)
/// * `buf` - Ponteiro para buffer
/// * `len` - Tamanho do buffer
///
/// # Returns
///
/// Número de bytes escritos, ou -1 em erro
fn syscall_write(fd: u64, buf: u64, len: u64) -> u64 {
    // Apenas stdout e stderr por enquanto
    if fd != STDOUT && fd != STDERR {
        return u64::MAX; // -1 (erro)
    }

    // Validar ponteiro e tamanho
    if buf == 0 || len == 0 || len > 1024 * 1024 {
        return u64::MAX; // -1 (erro)
    }

    // Criar slice do buffer
    let data = unsafe { core::slice::from_raw_parts(buf as *const u8, len as usize) };

    // Tentar converter para string UTF-8
    if let Ok(s) = core::str::from_utf8(data) {
        // Escrever no console serial
        serial::print(s);
        len // Retornar número de bytes escritos
    } else {
        // Não é UTF-8 válido, escrever bytes brutos como '?'
        for _ in data {
            serial::print("?");
        }
        len
    }
}

/// SYS_EXIT - Terminar processo
///
/// # Arguments
///
/// * `code` - Código de saída
///
/// # Returns
///
/// Nunca retorna (processo terminado)
fn syscall_exit(_code: u64) -> u64 {
    serial::println("[SYSCALL] Processo terminando");

    // TODO: Remover processo do ProcessManager
    // Por enquanto, apenas loop infinito
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
