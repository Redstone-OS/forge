//! # System Information Syscalls
//!
//! sysinfo, debug

use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// === WRAPPERS ===

pub fn sys_sysinfo_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_sysinfo(args.arg1, args.arg2)
}

pub fn sys_debug_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_debug(args.arg1 as u32, args.arg2, args.arg3)
}

// === IMPLEMENTAÇÕES ===

/// Obtém informações do sistema
///
/// # Args
/// - out_ptr: ponteiro para SysInfo de saída
/// - out_len: tamanho do buffer
///
/// # Returns
/// Bytes escritos ou erro
pub fn sys_sysinfo(out_ptr: usize, out_len: usize) -> SysResult<usize> {
    // TODO: Preencher SysInfo com dados reais
    // TODO: copy_to_user

    let _ = (out_ptr, out_len);
    Err(SysError::NotImplemented)
}

/// Comandos de debug
///
/// # Args
/// - cmd: comando (KPRINT, DUMP_REGS, etc)
/// - arg_ptr: ponteiro para argumento
/// - arg_len: tamanho do argumento
///
/// # Returns
/// Depende do comando
pub fn sys_debug(cmd: u32, arg_ptr: usize, arg_len: usize) -> SysResult<usize> {
    // Apenas em debug builds
    #[cfg(debug_assertions)]
    {
        match cmd {
            debug_cmd::KPRINT => {
                // TODO: Copiar string e imprimir
                let _ = (arg_ptr, arg_len);
                crate::kinfo!("(Debug) KPRINT chamado");
                return Ok(0);
            }
            debug_cmd::DUMP_REGS => {
                crate::kinfo!("(Debug) DUMP_REGS chamado");
                return Ok(0);
            }
            debug_cmd::BREAKPOINT => {
                crate::kinfo!("(Debug) BREAKPOINT chamado");
                unsafe { core::arch::asm!("int3") };
                return Ok(0);
            }
            _ => {}
        }
    }

    let _ = (cmd, arg_ptr, arg_len);
    Err(SysError::NotImplemented)
}

/// Comandos de debug
pub mod debug_cmd {
    pub const KPRINT: u32 = 0x01;
    pub const DUMP_REGS: u32 = 0x02;
    pub const DUMP_MEM: u32 = 0x03;
    pub const BREAKPOINT: u32 = 0x04;
}

/// Informações do sistema
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SysInfo {
    pub kernel_version: u32,
    pub abi_version: u32,
    pub total_memory: u64,
    pub free_memory: u64,
    pub uptime_ms: u64,
    pub num_cpus: u32,
    pub num_processes: u32,
}
