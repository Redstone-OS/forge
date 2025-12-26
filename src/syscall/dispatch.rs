//! Dispatcher Central de Syscalls
//!
//! Ponto de entrada único para todas as syscalls.
//! Roteia baseado no número da syscall para os módulos específicos.

use super::abi::SyscallArgs;
use super::error::{SysError, SysResult};
use super::numbers::*;
use crate::arch::x86_64::idt::ContextFrame;

/// Handler de syscalls chamado pelo Assembly (via int 0x80).
///
/// Extrai argumentos do contexto, despacha para o handler correto,
/// e coloca o resultado em RAX.
#[no_mangle]
pub extern "C" fn syscall_dispatcher(ctx: &mut ContextFrame) {
    let args = SyscallArgs::from_context(ctx);

    crate::ktrace!("(Syscall) num=", args.num);
    crate::klog!(" arg1=", args.arg1, " arg2=", args.arg2);
    crate::knl!();

    let result = dispatch(&args);

    // Resultado vai em RAX (positivo = sucesso, negativo = erro)
    ctx.rax = match result {
        Ok(val) => val as u64,
        Err(e) => {
            crate::ktrace!("(Syscall) Erro na syscall num=", args.num);
            e.as_isize() as u64
        }
    };
}

/// Despacha syscall para o handler correto.
fn dispatch(args: &SyscallArgs) -> SysResult<usize> {
    match args.num {
        // === Processo ===
        SYS_EXIT => super::process::sys_exit(args.arg1 as i32),
        SYS_SPAWN => super::process::sys_spawn(args.arg1, args.arg2, args.arg3, args.arg4),
        SYS_WAIT => super::process::sys_wait(args.arg1, args.arg2),
        SYS_YIELD => super::process::sys_yield(),
        SYS_GETPID => super::process::sys_getpid(),
        SYS_GETTASKINFO => super::process::sys_gettaskinfo(args.arg1, args.arg2),

        // === Memória ===
        SYS_ALLOC => super::memory::sys_alloc(args.arg1, args.arg2),
        SYS_FREE => super::memory::sys_free(args.arg1, args.arg2),
        SYS_MAP => super::memory::sys_map(args.arg1, args.arg2, args.arg3, args.arg4),
        SYS_UNMAP => super::memory::sys_unmap(args.arg1, args.arg2),

        // === Handles ===
        SYS_HANDLE_CREATE => {
            super::handle::sys_handle_create(args.arg1, args.arg2, args.arg3, args.arg4)
        }
        SYS_HANDLE_DUP => super::handle::sys_handle_dup(args.arg1, args.arg2),
        SYS_HANDLE_CLOSE => super::handle::sys_handle_close(args.arg1),
        SYS_CHECK_RIGHTS => super::handle::sys_check_rights(args.arg1, args.arg2),

        // === IPC ===
        SYS_CREATE_PORT => super::ipc::sys_create_port(args.arg1),
        SYS_SEND_MSG => super::ipc::sys_send_msg(args.arg1, args.arg2, args.arg3, args.arg4),
        SYS_RECV_MSG => super::ipc::sys_recv_msg(args.arg1, args.arg2, args.arg3, args.arg4),
        SYS_PEEK_MSG => super::ipc::sys_peek_msg(args.arg1, args.arg2, args.arg3),

        // === IO ===
        SYS_READV => super::io::sys_readv(args.arg1, args.arg2, args.arg3, args.arg4),
        SYS_WRITEV => super::io::sys_writev(args.arg1, args.arg2, args.arg3, args.arg4),

        // === Tempo ===
        SYS_CLOCK_GET => super::time::sys_clock_get(args.arg1, args.arg2),
        SYS_SLEEP => super::time::sys_sleep(args.arg1),
        SYS_MONOTONIC => super::time::sys_monotonic(),

        // === Async IO (futuro) ===
        SYS_CREATE_RING | SYS_SUBMIT_IO | SYS_WAIT_IO | SYS_CLOSE_RING => {
            crate::kwarn!("(Syscall) Async IO não implementado num=", args.num);
            Err(SysError::NotImplemented)
        }

        // === Sistema ===
        SYS_SYSINFO => super::system::sys_sysinfo(args.arg1, args.arg2),
        SYS_DEBUG => super::system::sys_debug(args.arg1, args.arg2, args.arg3),

        // === Desconhecida ===
        _ => {
            crate::kwarn!("(Syscall) Desconhecida num=", args.num);
            Err(SysError::NotImplemented)
        }
    }
}
