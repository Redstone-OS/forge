#![allow(dead_code)]
//! # Syscall Dispatcher
//!
//! Despachante baseado em tabela para despacho O(1).
//! Os logs de tracer estão desativados por padrão para evitar poluição.
//! Ative-os para depuração, desative quando puder.

pub mod table;

use super::abi::SyscallArgs;
use super::error::{SysError, SysResult};
use crate::arch::x86_64::idt::ContextFrame;

pub use table::SYSCALL_TABLE;

/// Handler de syscall (entry point do assembly)
///
/// Usa acesso volatile para evitar que o compilador gere código SSE
#[no_mangle]
#[inline(never)]
pub extern "C" fn syscall_dispatcher(ctx: *mut ContextFrame) {
    // Acesso via ponteiro bruto com volatile para evitar SSE
    unsafe {
        // crate::ktrace!("(Syscall) ENTRADA no dispatcher");
        // crate::ktrace!("(Syscall) ctx ptr=", ctx as u64);
        // NOTA: ktrace desativado para evitar overhead em loops rápidos

        // Ler argumentos da syscall
        let num = core::ptr::read_volatile(core::ptr::addr_of!((*ctx).rax)) as usize;
        let arg1 = core::ptr::read_volatile(core::ptr::addr_of!((*ctx).rdi)) as usize;
        let arg2 = core::ptr::read_volatile(core::ptr::addr_of!((*ctx).rsi)) as usize;
        let arg3 = core::ptr::read_volatile(core::ptr::addr_of!((*ctx).rdx)) as usize;
        let arg4 = core::ptr::read_volatile(core::ptr::addr_of!((*ctx).r10)) as usize;
        let arg5 = core::ptr::read_volatile(core::ptr::addr_of!((*ctx).r8)) as usize;
        let arg6 = core::ptr::read_volatile(core::ptr::addr_of!((*ctx).r9)) as usize;

        // crate::ktrace!("(Syscall) num=", num as u64);
        // crate::ktrace!("(Syscall) arg1=", arg1 as u64);
        // crate::ktrace!("(Syscall) arg2=", arg2 as u64);
        // NOTA: ktrace desativado para evitar overhead em loops rápidos

        // Construir struct de argumentos
        let args = SyscallArgs {
            num,
            arg1,
            arg2,
            arg3,
            arg4,
            arg5,
            arg6,
        };

        // Dispatch via tabela
        let result: u64 = if num < table::TABLE_SIZE {
            if let Some(handler) = SYSCALL_TABLE[num] {
                // crate::ktrace!("(Syscall) Handler encontrado");
                // NOTA: ktrace desativado para evitar overhead
                match handler(&args) {
                    Ok(val) => val as u64,
                    Err(e) => {
                        crate::kerror!("(Syscall) Handler retornou erro");
                        e.as_isize() as u64
                    }
                }
            } else {
                // Fallback para syscalls hardcoded (compatibilidade)
                dispatch_hardcoded(num, arg1, arg2)
            }
        } else {
            crate::ktrace!("(Syscall) num fora do range");
            (-1i64) as u64 // ENOSYS
        };

        // crate::ktrace!("(Syscall) Resultado=", result);
        // NOTA: ktrace desativado para evitar overhead

        // Escrever resultado em RAX via volatile
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*ctx).rax), result);

        // NOTA: NÃO chamar maybe_reschedule() aqui!
        // Context switch no meio do dispatcher corrompe o estado da task.
        // Preempção deve acontecer apenas em pontos seguros (yield explícito).

        // crate::ktrace!("(Syscall) SAINDO do dispatcher");
        // NOTA: ktrace desativado para evitar overhead
    }
}

/// Fallback para syscalls hardcoded (console, yield)
#[inline(always)]
unsafe fn dispatch_hardcoded(num: usize, arg1: usize, arg2: usize) -> u64 {
    match num {
        0xF3 => {
            crate::ktrace!("(Syscall) SYS_CONSOLE_WRITE (hardcoded)");
            // SYS_CONSOLE_WRITE - escrever na serial diretamente
            if arg1 != 0 && arg2 != 0 {
                for i in 0..arg2 {
                    let byte = core::ptr::read_volatile((arg1 + i) as *const u8);
                    core::arch::asm!(
                        "out dx, al",
                        in("dx") 0x3F8u16,
                        in("al") byte,
                        options(nostack, preserves_flags),
                    );
                }
                arg2 as u64
            } else {
                0
            }
        }
        0x04 => {
            // SYS_YIELD
            0
        }
        _ => {
            crate::ktrace!("(Syscall) syscall desconhecida!", num as u64);
            (-1i64) as u64 // ENOSYS
        }
    }
}

/// Dispatch via lookup table (versão safe para uso futuro)
fn dispatch(args: &SyscallArgs) -> SysResult<usize> {
    if args.num >= table::TABLE_SIZE {
        return Err(SysError::InvalidSyscall);
    }

    match SYSCALL_TABLE[args.num] {
        Some(handler) => handler(args),
        None => Err(SysError::NotImplemented),
    }
}
