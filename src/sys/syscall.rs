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
            // sys_write(fd, buffer, len)
            // fd: arg1 (1 = stdout)
            // buffer: arg2 (ptr)
            // len: arg3 (usize)
            let fd = arg1;
            let ptr = arg2 as *const u8;
            let len = arg3 as usize;

            if fd == 1 {
                // STDOUT
                // Validar ponteiro (básico)
                if ptr.is_null() {
                    context.rax = -1i64 as u64; // Error
                    return;
                }

                // Safety: Assumimos que o userspace passou ponteiro válido por enquanto.
                // Em OS real, precisa validar se endereços pertencem ao processo.
                let slice = unsafe { core::slice::from_raw_parts(ptr, len) };

                // Tenta converter para UTF-8 string, se falhar imprime bytes raw?
                // Vamos imprimir lossy.
                if let Ok(s) = core::str::from_utf8(slice) {
                    crate::kprint!("{}", s);
                } else {
                    for &b in slice {
                        crate::kprint!("{}", b as char);
                    }
                }

                context.rax = len as u64; // Retorna bytes escritos
            } else {
                context.rax = -1i64 as u64; // EBADF
            }
        }
        SYS_YIELD => {
            crate::sched::scheduler::yield_now();
            context.rax = 0;
        }
        _ => {
            crate::kprintln!("Syscall desconhecida: {}", syscall_num);
            context.rax = -1i64 as u64; // ENOSYS
        }
    }
}
