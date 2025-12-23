//! Dispatcher Central de Syscalls.
//!
//! Roteia o número da syscall para a implementação correta.

use super::numbers::*;
use super::{fs, process};

/// Roteia a syscall.
pub fn dispatch(num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    /* NOTA DE DEBUG:
       Descomente para ver todas as syscalls passando (muito verboso).
       crate::kinfo!("SYSCALL: #{} args({:x}, {:x}, {:x})", num, arg1, arg2, arg3);
    */

    match num {
        // Filesystem
        SYS_WRITE => fs::sys_write(arg1, arg2, arg3),
        SYS_READ => fs::sys_read(arg1, arg2, arg3),

        // Process
        SYS_EXIT => process::sys_exit(arg1 as i32),
        SYS_SCHED_YIELD => process::sys_yield(),
        SYS_GETPID => process::sys_getpid(),

        // Não implementado
        _ => {
            crate::kwarn!("Syscall #{} not implemented", num);
            ENOSYS
        }
    }
}
