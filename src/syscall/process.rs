//! Syscalls de Processo
//!
//! Gerenciamento de processos e escalonamento.

use super::error::{SysError, SysResult};
use crate::sched::scheduler::SCHEDULER;

/// Encerra o processo atual.
///
/// # Syscall
/// `SYS_EXIT (0x01)` - Args: (exit_code: i32)
///
/// # Comportamento
/// - Marca a tarefa como Terminated
/// - Remove da runqueue
/// - Cede CPU para sempre (não retorna)
pub fn sys_exit(code: i32) -> SysResult<usize> {
    crate::kinfo!("(Syscall) sys_exit: Processo encerrado com código {}", code);

    // TODO: Implementar exit real quando tiver process management
    // Por enquanto: loop infinito cedendo CPU
    loop {
        let _ = sys_yield();
        // Em um sistema real, o scheduler removeria essa task
    }
}

/// Cria um novo processo (spawn).
///
/// # Syscall
/// `SYS_SPAWN (0x02)` - Args: (image_ptr, image_len, args_ptr, args_len)
///
/// # TODO
/// Esta syscall será implementada quando tivermos:
/// - ELF loader no userspace
/// - Memory space isolation
/// - Process table
pub fn sys_spawn(
    _image_ptr: usize,
    _image_len: usize,
    _args_ptr: usize,
    _args_len: usize,
) -> SysResult<usize> {
    crate::kwarn!("(Syscall) sys_spawn não implementado");
    Err(SysError::NotImplemented)
}

/// Espera um processo filho terminar.
///
/// # Syscall
/// `SYS_WAIT (0x03)` - Args: (task_id, timeout_ms)
///
/// # TODO
/// Requer: process parent/child tracking, wait queues
pub fn sys_wait(_task_id: usize, _timeout_ms: usize) -> SysResult<usize> {
    crate::kwarn!("(Syscall) sys_wait não implementado");
    Err(SysError::NotImplemented)
}

/// Cede o restante do quantum de tempo.
///
/// # Syscall
/// `SYS_YIELD (0x04)` - Args: nenhum
///
/// # Retorno
/// Sempre retorna 0 (sucesso)
pub fn sys_yield() -> SysResult<usize> {
    // Solicitar troca de contexto
    let switch = {
        let mut sched = SCHEDULER.lock();
        sched.schedule()
    };

    if let Some((old_sp, new_sp)) = switch {
        unsafe {
            crate::sched::context_switch(old_sp as *mut u64, new_sp);
        }
    }

    Ok(0)
}

/// Obtém o PID do processo atual.
///
/// # Syscall
/// `SYS_GETPID (0x05)` - Args: nenhum
///
/// # Retorno
/// TaskId do processo atual
pub fn sys_getpid() -> SysResult<usize> {
    // TODO: Acessar current_task do scheduler
    // Por enquanto retorna 1 (init)
    Ok(1)
}

/// Obtém informações sobre uma tarefa.
///
/// # Syscall
/// `SYS_GETTASKINFO (0x06)` - Args: (task_id, out_ptr)
///
/// # TODO
/// Requer: struct TaskInfo, validação de ponteiro
pub fn sys_gettaskinfo(_task_id: usize, _out_ptr: usize) -> SysResult<usize> {
    Err(SysError::NotImplemented)
}
