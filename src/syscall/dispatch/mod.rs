//! # Syscall Dispatcher
//!
//! Table-based dispatcher para O(1) dispatch.

pub mod table;

use super::abi::SyscallArgs;
use super::error::{SysError, SysResult};
use crate::arch::x86_64::idt::ContextFrame;

pub use table::SYSCALL_TABLE;

/// Handler de syscall (entry point do assembly)
#[no_mangle]
pub extern "C" fn syscall_dispatcher(ctx: &mut ContextFrame) {
    let args = SyscallArgs::from_context(ctx);

    crate::ktrace!("(Syscall) num=", args.num);

    let result = dispatch(&args);

    // Resultado em RAX
    ctx.rax = match result {
        Ok(val) => val as u64,
        Err(e) => {
            crate::ktrace!("(Syscall) Erro num=", args.num);
            e.as_isize() as u64
        }
    };
}

/// Dispatch via lookup table
fn dispatch(args: &SyscallArgs) -> SysResult<usize> {
    if args.num >= table::TABLE_SIZE {
        return Err(SysError::InvalidSyscall);
    }

    match SYSCALL_TABLE[args.num] {
        Some(handler) => handler(args),
        None => {
            crate::kwarn!("(Syscall) Não implementada num=", args.num);
            Err(SysError::NotImplemented)
        }
    }
}
