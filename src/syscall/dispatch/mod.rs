#![allow(dead_code)]
//! # Syscall Dispatcher
//!
//! Table-based dispatcher para O(1) dispatch.

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
        crate::ktrace!("[SYSCALL] ENTRADA no dispatcher");
        crate::ktrace!("[SYSCALL] ctx ptr=", ctx as u64);

        // Ler o número da syscall
        let num = core::ptr::read_volatile(core::ptr::addr_of!((*ctx).rax)) as usize;
        let arg1 = core::ptr::read_volatile(core::ptr::addr_of!((*ctx).rdi)) as usize;
        let arg2 = core::ptr::read_volatile(core::ptr::addr_of!((*ctx).rsi)) as usize;

        crate::ktrace!("[SYSCALL] num=", num as u64);
        crate::ktrace!("[SYSCALL] arg1=", arg1 as u64);
        crate::ktrace!("[SYSCALL] arg2=", arg2 as u64);

        // Dispatch hardcoded
        let result: u64 = match num {
            0xF3 => {
                crate::ktrace!("[SYSCALL] SYS_CONSOLE_WRITE chamado");
                // SYS_CONSOLE_WRITE - escrever na serial diretamente
                if arg1 != 0 && arg2 != 0 {
                    crate::ktrace!("[SYSCALL] Escrevendo na serial...");
                    for i in 0..arg2 {
                        let byte = core::ptr::read_volatile((arg1 + i) as *const u8);
                        // Escrever diretamente na porta serial COM1 (0x3F8)
                        core::arch::asm!(
                            "out dx, al",
                            in("dx") 0x3F8u16,
                            in("al") byte,
                            options(nostack, preserves_flags),
                        );
                    }
                    crate::ktrace!("[SYSCALL] Escrita serial OK");
                    arg2 as u64
                } else {
                    crate::ktrace!("[SYSCALL] args inválidos");
                    0
                }
            }
            _ => {
                crate::ktrace!("[SYSCALL] syscall desconhecida!");
                (-1i64) as u64 // ENOSYS
            }
        };

        crate::ktrace!("[SYSCALL] Resultado=", result);

        // Escrever resultado em RAX via volatile
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*ctx).rax), result);

        crate::ktrace!("[SYSCALL] SAINDO do dispatcher");
    }
}

/// Dispatch via lookup table (não usada no teste mínimo)
fn dispatch(args: &SyscallArgs) -> SysResult<usize> {
    if args.num >= table::TABLE_SIZE {
        return Err(SysError::InvalidSyscall);
    }

    match SYSCALL_TABLE[args.num] {
        Some(handler) => handler(args),
        None => Err(SysError::NotImplemented),
    }
}
