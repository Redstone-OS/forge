//! Lógica de Troca de Contexto (Context Switching)

use crate::sched::task::context::{jump_to_context, switch, CpuContext};
use crate::sched::task::{Task, TaskState};
use crate::sync::SpinlockGuard;
use alloc::boxed::Box;
use core::pin::Pin;

/// Efetua a troca de contexto de baixo nível.
///
/// Esta função realiza a transição final de ownership e chama o assembly.
///
/// **Nota sobre o fluxo:** Se for uma troca entre tarefas existentes (`switch`),
/// esta função IRÁ RETORNAR quando a tarefa atual for re-escalonada no futuro.
/// Se for um salto para uma tarefa nova (`jump_to_context`), ela não retorna.
///  
/// # Safety
/// Deve ser chamada com interrupções desabilitadas.
pub unsafe fn prepare_and_switch_to(
    mut next: Pin<Box<Task>>,
    old_ctx: Option<*mut CpuContext>,
    mut current_guard: SpinlockGuard<Option<Pin<Box<Task>>>>,
) {
    // Extrair dados necessários
    let is_new = next.state == TaskState::Created;
    let new_ctx_ptr = &next.context as *const _;

    // Marcar nova task como Running
    core::pin::Pin::get_unchecked_mut(next.as_mut()).state = TaskState::Running;

    // Log de troca
    crate::ktrace!("(Sched) Mudando para PID:", next.tid.as_u32() as u64);

    // Aplicar estado de hardware (GDT, CR3)
    next.apply_hardware_state();

    // Transferir ownership para o global CURRENT
    *current_guard = Some(next);
    drop(current_guard);

    // Efetuar o salto/troca final
    if let Some(old_ctx_ptr) = old_ctx {
        // Troca completa (salva atual, restaura próxima)
        // Quando esta tarefa for retomada, ela voltará exatamente aqui.
        switch(&mut *old_ctx_ptr, &*new_ctx_ptr);
        // Task retomada - continua execução normal
    } else {
        // Apenas restaura próxima (sem contexto anterior para salvar)
        if is_new {
            // Task nova - salta para entry point
            jump_to_context(&*new_ctx_ptr);
        } else {
            // Task existente sem old_ctx - isso só deveria acontecer na idle task
            // ou na primeira execução do scheduler
            let mut dummy = CpuContext::new();
            switch(&mut dummy, &*new_ctx_ptr);
        }
    }
}
