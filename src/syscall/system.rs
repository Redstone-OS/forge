//! # System Info & Debug Syscalls
//!
//! Metadados globais e ferramentas de diagnóstico.
//!
//! ## 🎯 Propósito
//! - **Introspection:** `sysinfo` permite que ferramentas (ex: `top`, `neofetch`) saibam o estado da máquina.
//! - **Debug:** `sys_debug` é uma "porta dos fundos" controlada para logs e diagnósticos durante desenvolvimento.
//!
//! ## 🏗️ Arquitetura
//! - **Struct Stability:** `SysInfo` é `#[repr(C)]` para garantir layout fixo entre versões.
//! - **Debug Channel:** `sys_debug` bypassa abstrações de arquivo para garantir que logs saiam mesmo se o VFS quebrar.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Missing Metrics:** `SysInfo` tem placeholders (hardcoded 512MB RAM). O userspace não tem visão real de consumo de memória.
//! - **Security Risk:** `sys_debug` deve ser desabilitado ou restrito em builds `RELEASE`. Qualquer processo pode spammar o log do kernel (DoS).
//!
//! ## 🛠️ TODOs
//! - [ ] **TODO: (Security)** Restringir `SYS_DEBUG` apenas para **Development Mode** ou Capability de Admin.
//! - [ ] **TODO: (Feature)** Conectar `SysInfo` aos contadores reais do PMM e Scheduler.
//!
use super::error::{SysError, SysResult};
use super::numbers::debug_cmd;
use core::slice;

/// Informações do sistema.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SysInfo {
    /// Versão major do kernel
    pub version_major: u16,
    /// Versão minor do kernel
    pub version_minor: u16,
    /// Versão patch do kernel
    pub version_patch: u16,
    /// Flags (reservado)
    pub flags: u16,
    /// Memória total em bytes
    pub total_memory: u64,
    /// Memória livre em bytes
    pub free_memory: u64,
    /// Número de CPUs
    pub cpu_count: u32,
    /// Uptime em segundos
    pub uptime_seconds: u32,
    /// Número de processos ativos
    pub process_count: u32,
    /// Padding
    pub _reserved: u32,
}

impl SysInfo {
    pub const fn new() -> Self {
        Self {
            version_major: 0,
            version_minor: 1,
            version_patch: 0,
            flags: 0,
            total_memory: 0,
            free_memory: 0,
            cpu_count: 1,
            uptime_seconds: 0,
            process_count: 0,
            _reserved: 0,
        }
    }
}

/// Obtém informações do sistema.
///
/// # Syscall
/// `SYS_SYSINFO (0xF0)` - Args: (out_ptr, out_len)
///
/// # Argumentos
/// - `out_ptr`: Ponteiro para SysInfo
/// - `out_len`: Tamanho do buffer (deve ser >= sizeof(SysInfo))
///
/// # Retorno
/// Bytes escritos
pub fn sys_sysinfo(out_ptr: usize, out_len: usize) -> SysResult<usize> {
    let sysinfo_size = core::mem::size_of::<SysInfo>();

    if out_ptr == 0 {
        return Err(SysError::BadAddress);
    }

    if out_len < sysinfo_size {
        return Err(SysError::InvalidArgument);
    }

    // Coletar informações
    let mut info = SysInfo::new();

    // TODO: Obter da MM real
    info.total_memory = 512 * 1024 * 1024; // 512MB placeholder
    info.free_memory = 256 * 1024 * 1024; // placeholder

    // Uptime
    let ticks = super::time::get_ticks();
    info.uptime_seconds = (ticks / 100) as u32; // Assumindo 100Hz

    // TODO: Contar processos reais
    info.process_count = 1;

    // Escrever resultado
    unsafe {
        let ptr = out_ptr as *mut SysInfo;
        *ptr = info;
    }

    Ok(sysinfo_size)
}

/// Comandos de debug.
///
/// # Syscall
/// `SYS_DEBUG (0xFF)` - Args: (cmd, arg_ptr, arg_len)
///
/// # Comandos
/// - KPRINT: Imprime string no log do kernel
/// - DUMP_REGS: Dump de registradores (não implementado)
/// - DUMP_MEM: Dump de memória (não implementado)
/// - BREAKPOINT: Dispara breakpoint
pub fn sys_debug(cmd: usize, arg_ptr: usize, arg_len: usize) -> SysResult<usize> {
    match cmd as u32 {
        debug_cmd::KPRINT => {
            // Imprimir string
            if arg_ptr == 0 || arg_len == 0 {
                return Ok(0);
            }

            // Limite de segurança
            let len = arg_len.min(1024);

            // TODO: Validar ponteiro userspace
            let data = unsafe { slice::from_raw_parts(arg_ptr as *const u8, len) };

            match core::str::from_utf8(data) {
                Ok(s) => {
                    crate::kinfo!("(Debug) ");
                    crate::klog!(s);
                    crate::knl!();
                }
                Err(_) => crate::kwarn!("(Debug) sys_debug: Dados não-UTF8 recebidos"),
            }

            Ok(len)
        }

        debug_cmd::DUMP_REGS => {
            crate::kinfo!("(Debug) sys_debug: DUMP_REGS não implementado");
            Err(SysError::NotImplemented)
        }

        debug_cmd::DUMP_MEM => {
            crate::kinfo!("(Debug) sys_debug: DUMP_MEM não implementado");
            Err(SysError::NotImplemented)
        }

        debug_cmd::BREAKPOINT => {
            crate::kinfo!("(Debug) sys_debug: Breakpoint acionado por userspace");
            unsafe {
                core::arch::asm!("int 3");
            }
            Ok(0)
        }

        _ => {
            crate::kwarn!("(Debug) sys_debug: Comando desconhecido cmd=", cmd as u64);
            Err(SysError::InvalidArgument)
        }
    }
}
