//! Lógica central do Scheduler
//!
//! Contém o loop principal, troca de contexto e gerenciamento de estado global.

use crate::arch::Cpu;
use crate::sched::task::Task;
use crate::sched::task::TaskState;
use crate::sync::Spinlock;
use alloc::boxed::Box;
use core::pin::Pin;

use super::runqueue::RUNQUEUE;

/// Task atualmente executando
/// TODO: Tornar per-cpu quando tivermos suporte a SMP
pub static CURRENT: Spinlock<Option<Pin<Box<Task>>>> = Spinlock::new(None);

// NOTA: Preempção diferida (NEED_RESCHED) removida temporariamente.
// Para preempção real, precisamos modificar o assembly do timer handler
// para salvar contexto completo antes de verificar a flag.
// Atualmente usamos cooperative multitasking via yield_now().

/// Inicializa o scheduler
pub fn init() {
    crate::kinfo!("(Sched) Inicializando scheduler...");
    crate::kinfo!("(Sched) Scheduler inicializado. Idle loop integrado.");
}

/// Retorna ponteiro para a task atual (unsafe se dereferenciado sem lock, uteil para IDs)
pub fn current() -> Option<*const Task> {
    CURRENT
        .lock()
        .as_ref()
        .map(|t| t.as_ref().get_ref() as *const Task)
}

/// Adiciona task à fila de execução
pub fn enqueue(task: Pin<Box<Task>>) {
    RUNQUEUE.lock().push(task);
}

/// Seleciona próxima task para executar
pub fn pick_next() -> Option<Pin<Box<Task>>> {
    RUNQUEUE.lock().pop()
}

/// Yield: cede CPU voluntariamente
pub fn yield_now() {
    Cpu::disable_interrupts();
    schedule();
    Cpu::enable_interrupts();
}

/// Libera o lock do scheduler manualmente (usado por new tasks)
/// # Safety
/// Somente chamar no início de novas tasks.
#[no_mangle]
pub unsafe extern "C" fn release_scheduler_lock() {
    CURRENT.force_unlock();
}

/// Exit: termina processo atual e pula para próximo
pub fn exit_current() -> ! {
    Cpu::disable_interrupts();

    // 1. Remover processo atual do CURRENT
    {
        let mut current_guard = CURRENT.lock();
        if let Some(old_task) = current_guard.take() {
            // Move para lista de zumbis para limpeza posterior
            crate::sched::task::lifecycle::add_zombie(old_task);
        }
    }

    // 2. Schedule next
    if let Some(mut next) = pick_next() {
        let mut current_guard = CURRENT.lock();

        // Marcar nova task como Running
        unsafe { Pin::get_unchecked_mut(next.as_mut()) }.state = TaskState::Running;

        let next_ref = next.as_ref();
        let ctx_ptr = &{ core::pin::Pin::get_ref(next_ref) }.context
            as *const crate::sched::task::context::CpuContext;
        let new_cr3 = { core::pin::Pin::get_ref(next_ref) }.cr3;
        let kernel_stack = { core::pin::Pin::get_ref(next_ref) }.kernel_stack.as_u64();

        *current_guard = Some(next);
        drop(current_guard);

        unsafe {
            if kernel_stack != 0 {
                crate::arch::x86_64::gdt::set_kernel_stack(kernel_stack);
                crate::arch::x86_64::syscall::set_kernel_rsp(kernel_stack);
            }
            if new_cr3 != 0 {
                core::arch::asm!("mov cr3, {}", in(reg) new_cr3);
            }
            crate::sched::task::context::jump_to_context(&*ctx_ptr);
        }
    } else {
        // Sem tasks, halt
        loop {
            Cpu::enable_interrupts();
            Cpu::halt();
            Cpu::disable_interrupts();
        }
    }
}

/// Função principal de escalonamento
pub fn schedule() {
    let next = match pick_next() {
        Some(t) => t,
        None => return,
    };

    let mut current_guard = CURRENT.lock();
    if let Some(ref mut _current) = *current_guard {
        // Switch de A -> B
        let old_task = current_guard.take().unwrap();

        let mut old_task_pin = old_task;

        // Marcar task antiga como Ready (vai voltar pra fila)
        unsafe { Pin::get_unchecked_mut(old_task_pin.as_mut()) }.state = TaskState::Ready;

        // SAFETY: Temos ownership exclusivo via lock
        let old_ctx_ptr =
            &mut unsafe { Pin::get_unchecked_mut(old_task_pin.as_mut()) }.context as *mut _;

        // Marcar nova task como Running
        let mut next_mut = next;
        unsafe { Pin::get_unchecked_mut(next_mut.as_mut()) }.state = TaskState::Running;
        let new_ctx_ptr = &{ Pin::get_ref(next_mut.as_ref()) }.context as *const _;

        let new_cr3 = { Pin::get_ref(next_mut.as_ref()) }.cr3;

        // Devolve old task pra fila
        RUNQUEUE.lock().push(old_task_pin);

        // Seta nova task
        *current_guard = Some(next_mut);

        unsafe {
            if let Some(current_task) = current_guard.as_ref() {
                let stack_top = current_task.as_ref().kernel_stack.as_u64();
                if stack_top != 0 {
                    crate::arch::x86_64::gdt::set_kernel_stack(stack_top);
                    crate::arch::x86_64::syscall::set_kernel_rsp(stack_top);
                }
            }
            if new_cr3 != 0 {
                core::arch::asm!("mov cr3, {}", in(reg) new_cr3);
            }
            crate::sched::task::context::switch(&mut *old_ctx_ptr, &*new_ctx_ptr);
        }
    } else {
        // Startup/Idle -> B (Primeira task)
        crate::ktrace!("(Sched) Primeira task, usando jump_to_context");

        // Marcar nova task como Running primeiro
        let mut next_mut = next;
        unsafe { Pin::get_unchecked_mut(next_mut.as_mut()) }.state = TaskState::Running;

        // Agora extrair ponteiros
        let next_ref = next_mut.as_ref();
        let ctx_ptr =
            &{ Pin::get_ref(next_ref) }.context as *const crate::sched::task::context::CpuContext;
        let new_cr3 = { Pin::get_ref(next_ref) }.cr3;
        let kernel_stack = { Pin::get_ref(next_ref) }.kernel_stack.as_u64();

        *current_guard = Some(next_mut);
        drop(current_guard);

        unsafe {
            if kernel_stack != 0 {
                crate::arch::x86_64::gdt::set_kernel_stack(kernel_stack);
                crate::arch::x86_64::syscall::set_kernel_rsp(kernel_stack);
            }
            if new_cr3 != 0 {
                core::arch::asm!("mov cr3, {}", in(reg) new_cr3);
            }
            crate::sched::task::context::jump_to_context(&*ctx_ptr);
        }
    }
}

/// Loop principal do scheduler
///
/// Fica em loop chamando schedule() e entrando em halt quando não há tasks.
/// No idle, também limpa tasks zombies.
pub fn run() -> ! {
    Cpu::disable_interrupts();
    loop {
        schedule();
        if RUNQUEUE.lock().is_empty() {
            // Aproveita idle time para limpar zombies
            crate::sched::task::lifecycle::cleanup_all();

            Cpu::enable_interrupts();
            Cpu::halt();
            Cpu::disable_interrupts();
        }
    }
}
