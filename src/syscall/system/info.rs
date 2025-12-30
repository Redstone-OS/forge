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

    unsafe {
        // Método 1: Keyboard controller reset (PS/2)
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

        // Enviar comando de reset (0xFE) para o keyboard controller
        core::arch::asm!(
            "out 0x64, al",
            in("al") 0xFEu8,
            options(nostack, preserves_flags)
        );

        // Pequeno delay
        for _ in 0..100000u32 {
            core::arch::asm!("nop");
        }

        // Método 2: Triple Fault (fallback mais confiável)
        // Carregar IDT inválida e causar uma exceção
        crate::kinfo!("(Syscall) sys_reboot: Usando Triple Fault...");

        // Criar uma IDT nula (inválida)
        let null_idt: [u8; 6] = [0; 6]; // limit=0, base=0
        core::arch::asm!(
            "lidt [{}]",
            "int3",  // Causar exceção com IDT inválida = Triple Fault
            in(reg) &null_idt,
            options(nostack)
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

    unsafe {
        // Método 1: QEMU ISA debug exit device (port 0x501)
        // Este é configurável com -device isa-debug-exit
        core::arch::asm!(
            "out dx, al",
            in("dx") 0x501u16,
            in("al") 0x31u8,  // Exit code
            options(nostack, preserves_flags)
        );

        // Método 2: QEMU shutdown via port 0x604 (ACPI PM1a_CNT)
        core::arch::asm!(
            "out dx, ax",
            in("dx") 0x604u16,
            in("ax") 0x2000u16,  // SLP_TYP = 5 (S5 state) | SLP_EN = 1
            options(nostack, preserves_flags)
        );

        // Método 3: Bochs/QEMU shutdown string via port 0x8900
        let shutdown_str: &[u8; 8] = b"Shutdown";
        for byte in shutdown_str {
            core::arch::asm!(
                "out dx, al",
                in("dx") 0x8900u16,
                in("al") *byte,
                options(nostack, preserves_flags)
            );
        }

        // Método 4: VirtualBox ACPI shutdown
        core::arch::asm!(
            "out dx, ax",
            in("dx") 0x4004u16,
            in("ax") 0x3400u16,
            options(nostack, preserves_flags)
        );
    }

    // Fallback: halt infinito
    crate::kinfo!("(Syscall) sys_poweroff: Halt...");
    loop {
        unsafe { core::arch::asm!("cli; hlt") };
    }
}

/// Escreve na console (framebuffer + serial)
///
/// VERSÃO MÍNIMA: Usa inline assembly direto para evitar qualquer
/// código Rust que possa gerar instruções SSE/AVX
pub fn sys_console_write(buf_ptr: usize, len: usize) -> SysResult<usize> {
    if buf_ptr == 0 || len == 0 {
        return Ok(0);
    }

    // Limitar tamanho
    let safe_len = if len > 4096 { 4096 } else { len };

    // Escrever diretamente na porta serial COM1 (0x3F8) usando inline assembly
    // Isso evita chamar qualquer função Rust que possa usar SSE
    for i in 0..safe_len {
        let byte: u8 = unsafe { core::ptr::read_volatile((buf_ptr + i) as *const u8) };

        // Esperar que TX esteja vazio (bit 5 de Line Status Register)
        // Port 0x3FD = 0x3F8 + 5
        unsafe {
            core::arch::asm!(
                "mov dx, 0x3FD", // Line Status Register
                "2:",
                "in al, dx",
                "test al, 0x20", // Empty Transmitter Holding Register
                "jz 2b",
                "mov dx, 0x3F8", // Data Register
                "mov al, {byte}",
                "out dx, al",
                byte = in(reg_byte) byte,
                out("al") _,
                out("dx") _,
                options(nostack, preserves_flags),
            );
        }
    }

    Ok(safe_len)
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
