use crate::arch::x86_64::idt::ContextFrame;
use core::ffi::c_void;

// Syscall Numbers
const SYS_WRITE: u64 = 1;
const SYS_YIELD: u64 = 158; // Exemplo

/// Handler de Syscalls chamado pelo Assembly.
///
/// # Arguments
/// * `context`: Ponteiro para o stack frame com todos registradores salvos.
#[no_mangle]
pub extern "C" fn syscall_dispatcher(context: &mut ContextFrame) {
    let syscall_num = context.rax;
    let arg1 = context.rdi;
    let arg2 = context.rsi;
    let arg3 = context.rdx;

    match syscall_num {
        SYS_WRITE => {
            let fd = arg1;
            let ptr = arg2 as *const u8;
            let len = arg3 as usize;

            crate::ktrace!("(Sys) sys_write: fd={} ptr={:p} len={}", fd, ptr, len);

            if fd == 1 {
                // STDOUT
                if ptr.is_null() {
                    crate::kwarn!("(Sys) sys_write: Ponteiro nulo recebido");
                    context.rax = -1i64 as u64;
                    return;
                }

                let slice = unsafe { core::slice::from_raw_parts(ptr, len) };

                // Tenta converter para UTF-8 string
                if let Ok(s) = core::str::from_utf8(slice) {
                    crate::kprint!("{}", s);
                } else {
                    for &b in slice {
                        crate::kprint!("{}", b as char);
                    }
                }

                context.rax = len as u64; // Retorna bytes escritos
            } else {
                crate::kdebug!("(Sys) sys_write: FD {} não suportado", fd);
                context.rax = -1i64 as u64; // EBADF
            }
        }
        SYS_YIELD => {
            crate::ktrace!("(Sys) sys_yield: Cedendo tempo de CPU voluntariamente");
            crate::sched::scheduler::yield_now();
            context.rax = 0;
        }
        _ => {
            crate::kwarn!(
                "(Sys) Chamada inesperada: syscall {} não implementada",
                syscall_num
            );
            context.rax = -1i64 as u64; // ENOSYS
        }
    }
}
