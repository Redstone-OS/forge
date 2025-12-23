//! Implementação de Syscalls de Processos.

use super::numbers::*;
use crate::sched::scheduler::SCHEDULER;
use crate::sched::task::TaskState;

/// Encerra o processo atual.
pub fn sys_exit(code: i32) -> ! {
    crate::kinfo!("Process exiting with code: {}", code);

    // 1. Marcar tarefa atual como terminada
    {
        // Precisamos de acesso ao scheduler/task atual
        // Como o SCHEDULER é global, podemos alterar o estado.
        // Nota: Idealmente teríamos um método `scheduler.exit_current(code)`.

        // HACK TEMPORÁRIO: Como não expusemos `exit_current` no Scheduler da Fase 7,
        // vamos apenas logar e travar/yieldar loop.
        // Na implementação real, isso deve remover a task da runqueue.
    }

    // 2. Ceder CPU para sempre
    loop {
        sys_yield();
    }
}

/// Cede o restante do quantum de tempo (Voluntary Preemption).
pub fn sys_yield() -> isize {
    // Forçar agendamento
    // Nota: A função schedule() deve ser segura de chamar daqui.
    // Como estamos "dentro" do kernel via syscall (int 0x80), interrupções podem estar
    // desabilitadas ou habilitadas dependendo do stub.
    // O stub `int 0x80` desabilita IRQs (Interrupt Gate).

    let switch = {
        let mut sched = SCHEDULER.lock();
        sched.schedule()
    };

    if let Some((old_sp, new_sp)) = switch {
        unsafe {
            // Reutiliza o assembly de switch
            crate::sched::context_switch(old_sp as *mut u64, new_sp);
        }
    }

    0 // Sucesso
}

/// Retorna o ID do processo atual.
pub fn sys_getpid() -> isize {
    // TODO: Implementar quando Task tiver acesso fácil ao próprio ID via thread-local ou similar
    1
}
