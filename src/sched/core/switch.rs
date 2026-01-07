//! Lógica de Troca de Contexto (Context Switching)
//!
//! Helpers para context switch. O switch principal está em scheduler.rs.

use crate::sched::core::per_cpu::{this_cpu_id, CPUS};
use crate::sched::task::context::{jump_to_context, switch, CpuContext};
use crate::sched::task::{Task, TaskState};
use alloc::boxed::Box;
use core::pin::Pin;

/// Efetua a troca de contexto para uma nova task.
///
/// Esta função é usada quando não se está dentro do schedule() principal.
///
/// # Safety
/// Deve ser chamada com interrupções desabilitadas.
pub unsafe fn prepare_and_switch_to(mut next: Pin<Box<Task>>, old_ctx: Option<*mut CpuContext>) {
    let cpu_id = this_cpu_id();

    // Extrair dados necessários
    let is_new = next.state == TaskState::Created;
    let new_ctx_ptr = &next.context as *const _;

    // Marcar nova task como Running
    Pin::get_unchecked_mut(next.as_mut()).state = TaskState::Running;

    // Aplicar estado de hardware (GDT, CR3)
    next.apply_hardware_state();

    // Transferir ownership para CPUS[cpu_id].current
    {
        let mut guard = CPUS[cpu_id].lock();
        if let Some(ref mut cpu_data) = *guard {
            cpu_data.current = Some(next);
        }
    }

    // Efetuar o salto/troca final
    if let Some(old_ctx_ptr) = old_ctx {
        switch(&mut *old_ctx_ptr, &*new_ctx_ptr);
    } else {
        if is_new {
            jump_to_context(&*new_ctx_ptr);
        } else {
            let mut dummy = CpuContext::new();
            switch(&mut dummy, &*new_ctx_ptr);
        }
    }
}
