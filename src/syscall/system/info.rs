//! # System Information Syscalls
//!
//! sysinfo, debug, reboot, poweroff, console I/O

use crate::syscall::abi::SyscallArgs;
use crate::syscall::error::{SysError, SysResult};

// === WRAPPERS ===

pub fn sys_sysinfo_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_sysinfo(args.arg1, args.arg2)
}

pub fn sys_debug_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_debug(args.arg1 as u32, args.arg2, args.arg3)
}

pub fn sys_reboot_wrapper(_args: &SyscallArgs) -> SysResult<usize> {
    sys_reboot()
}

pub fn sys_poweroff_wrapper(_args: &SyscallArgs) -> SysResult<usize> {
    sys_poweroff()
}

pub fn sys_console_write_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_console_write(args.arg1, args.arg2)
}

pub fn sys_console_read_wrapper(args: &SyscallArgs) -> SysResult<usize> {
    sys_console_read(args.arg1, args.arg2)
}

// === IMPLEMENTAÇÕES ===

/// Obtém informações do sistema
pub fn sys_sysinfo(out_ptr: usize, out_len: usize) -> SysResult<usize> {
    let _ = (out_ptr, out_len);
    Err(SysError::NotImplemented)
}

/// Comandos de debug
pub fn sys_debug(cmd: u32, arg_ptr: usize, arg_len: usize) -> SysResult<usize> {
    #[cfg(debug_assertions)]
    {
        match cmd {
            debug_cmd::KPRINT => {
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

/// Reinicia o sistema
pub fn sys_reboot() -> SysResult<usize> {
    crate::kinfo!("(Syscall) sys_reboot: Reiniciando sistema...");

    // Keyboard controller reset (PS/2)
    unsafe {
        // Aguardar controller pronto
        let mut timeout = 100000u32;
        while timeout > 0 {
            let status: u8;
            core::arch::asm!(
                "in al, 0x64",
                out("al") status,
                options(nostack, preserves_flags)
            );
            if (status & 0x02) == 0 {
                break;
            }
            timeout -= 1;
        }

        // Enviar comando de reset (0xFE)
        core::arch::asm!(
            "out 0x64, al",
            in("al") 0xFEu8,
            options(nostack, preserves_flags)
        );
    }

    // Nunca deve chegar aqui
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

/// Desliga o sistema
pub fn sys_poweroff() -> SysResult<usize> {
    crate::kinfo!("(Syscall) sys_poweroff: Desligando sistema...");

    // QEMU shutdown via port 0x604 (ACPI)
    unsafe {
        core::arch::asm!(
            "out dx, ax",
            in("dx") 0x604u16,
            in("ax") 0x2000u16,
            options(nostack, preserves_flags)
        );
    }

    // Fallback: halt
    loop {
        unsafe { core::arch::asm!("cli; hlt") };
    }
}

/// Escreve na console (framebuffer + serial)
pub fn sys_console_write(buf_ptr: usize, len: usize) -> SysResult<usize> {
    if buf_ptr == 0 || len == 0 {
        return Ok(0);
    }

    // TODO: Validar ponteiro com copy_from_user
    let slice = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };

    // Escreve no framebuffer (console gráfico)
    crate::drivers::video::console_write_bytes(slice);

    // Também escreve na serial (para debug)
    for &byte in slice {
        crate::drivers::serial::emit(byte);
    }

    Ok(len)
}

/// Lê da console (serial) - blocking
pub fn sys_console_read(buf_ptr: usize, max_len: usize) -> SysResult<usize> {
    if buf_ptr == 0 || max_len == 0 {
        return Ok(0);
    }

    // TODO: Implementar leitura real da serial
    // Por agora retorna 0 (nenhum dado disponível)
    let _ = (buf_ptr, max_len);
    Ok(0)
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
