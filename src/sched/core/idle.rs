//! Idle Task - Tarefa ociosa dedicada para quando não há trabalho
//!
//! Esta módulo implementa uma Idle Task com TID 0 que é executada quando
//! não há nenhuma outra tarefa pronta. Isso resolve o problema de context
//! switching do idle loop, onde contextos eram perdidos.

use super::runqueue::RUNQUEUE;
use super::scheduler::CURRENT;
use crate::arch::Cpu;
use crate::mm::VirtAddr;
use crate::sched::task::context::CpuContext;
use crate::sched::task::{Task, TaskState};
use crate::sys::types::Tid;
use alloc::boxed::Box;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};

/// Flag indicando se a idle task foi inicializada
static IDLE_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Verifica se há tasks prontas para executar
fn has_ready_tasks() -> bool {
    if let Some(rq) = RUNQUEUE.try_lock() {
        !rq.queue.is_empty()
    } else {
        false
    }
}

/// Loop principal da idle task
///
/// Esta função é o entry point da idle task. Ela:
/// 1. Fica em loop infinito fazendo HLT (economia de energia)
/// 2. Verifica periodicamente se há tasks prontas
/// 3. Quando uma task está pronta, chama schedule() para trocar
#[no_mangle]
pub extern "C" fn idle_task_entry() -> ! {
    crate::kinfo!("(Idle) Idle task iniciada (TID 0)");

    let mut idle_count: u64 = 0;

    loop {
        // Habilita interrupções e espera próximo evento
        Cpu::enable_interrupts();
        Cpu::halt();
        Cpu::disable_interrupts();

        idle_count = idle_count.wrapping_add(1);

        // Log periódico a cada 1000 iterações (mais silencioso)
        if idle_count % 1000 == 0 {
            crate::kdebug!("(Idle) Ciclos:", idle_count);
        }

        // Verifica se há tasks prontas
        if has_ready_tasks() {
            // Há trabalho a fazer! Chama o scheduler
            super::scheduler::schedule();
            // Quando retornarmos aqui, significa que não há mais tasks prontas
        }
    }
}

/// Cria e inicializa a idle task
///
/// Deve ser chamada uma vez durante a inicialização do scheduler,
/// ANTES de qualquer outra task ser criada.
pub fn init_idle_task() {
    if IDLE_INITIALIZED.swap(true, Ordering::SeqCst) {
        crate::kwarn!("(Idle) init_idle_task chamado mais de uma vez!");
        return;
    }

    crate::kinfo!("(Idle) Criando idle task...");

    // Aloca stack para a idle task (16KB)
    let stack_size = 16 * 1024;
    let stack_layout = alloc::alloc::Layout::from_size_align(stack_size, 16).unwrap();
    let stack_ptr = unsafe { alloc::alloc::alloc_zeroed(stack_layout) };

    if stack_ptr.is_null() {
        panic!("(Idle) Falha ao alocar stack para idle task!");
    }

    let stack_top = (stack_ptr as u64) + stack_size as u64;

    // Cria a task manualmente (não usa NEXT_TID pois queremos TID 0)
    let mut name_buf = [0u8; 32];
    let name = b"idle";
    name_buf[..4].copy_from_slice(name);

    let mut idle_task = Box::pin(Task {
        tid: Tid::new(0),
        state: TaskState::Created,
        context: CpuContext::new(),
        kernel_stack: VirtAddr::new(stack_top),
        user_stack: VirtAddr::new(0),
        aspace: None,  // Idle task usa espaço de endereçamento do kernel
        priority: 255, // Menor prioridade possível
        accounting: crate::sched::task::accounting::Accounting::new(),
        parent_id: None,
        exit_code: None,
        pending_signals: 0,
        blocked_signals: 0,
        name: name_buf,
        handle_table: crate::syscall::handle::table::HandleTable::new(),
        wake_at: None,
        heap_start: 0,
        heap_next: 0,
    });

    // Configura o contexto para iniciar em idle_task_entry
    unsafe {
        let task_mut = Pin::get_unchecked_mut(idle_task.as_mut());
        task_mut.context.setup(
            VirtAddr::new(idle_task_entry as *const () as u64),
            VirtAddr::new(stack_top),
        );
        task_mut.state = TaskState::Running; // Idle task começa rodando
    }

    crate::kdebug!("(Idle) Idle task criada com TID 0");

    // Coloca a idle task em CURRENT para iniciar o sistema
    {
        let mut current = CURRENT.lock();
        *current = Some(idle_task);
    }

    crate::kdebug!("(Idle) Idle task instalada em CURRENT");
}

/// Verifica se a task atual é a idle task (TID 0)
pub fn is_idle_current() -> bool {
    if let Some(current) = CURRENT.try_lock() {
        if let Some(ref task) = *current {
            return task.tid.as_u32() == 0;
        }
    }
    false
}
