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

/// Sleep: coloca a task atual em estado dormente por N milissegundos
pub fn sleep_current(ms: u64) {
    if ms == 0 {
        yield_now();
        return;
    }

    Cpu::disable_interrupts();

    // 1. Pegar a task atual (take de CURRENT)
    let task = {
        let mut current_guard = CURRENT.lock();
        current_guard.take()
    };

    if let Some(mut task) = task {
        // 2. Calcular jiffies de despertar
        let now = crate::core::time::jiffies::get_jiffies();
        let ticks = crate::core::time::jiffies::millis_to_jiffies(ms);
        task.wake_at = Some(now + ticks);

        // 3. Mudar estado para Blocked
        task.state = TaskState::Blocked;

        crate::kdebug!("(Sched) Task PID: dormindo por", ms);

        // 4. Adicionar à SleepQueue
        super::sleep_queue::add_task(task);

        // 5. Chamar schedule() para rodar a próxima
        // Nota: as interrupções continuam desabilitadas até o switch terminar
        schedule();
    }

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
    let mut next = match pick_next() {
        Some(t) => t,
        // Se não houver tasks, precisamos ver se ainda temos o CURRENT rodando.
        // Se CURRENT for None (ex: dormindo) e RunQueue vazia, caímos no idle loop via run().
        None => return,
    };

    let mut current_guard = CURRENT.lock();

    // 1. Decidir o que fazer com a task ANTIGA
    if let Some(mut old_task) = current_guard.take() {
        // Se a task antiga estava Running, ela volta pra fila (preempção cooperativa)
        if old_task.state == TaskState::Running {
            unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;

            // Salvar contexto e trocar
            let old_ctx_ptr = unsafe {
                Pin::get_unchecked_mut(old_task.as_mut()) as *mut Task
                    as *mut crate::sched::task::context::CpuContext
            };

            // Devolver para a fila
            RUNQUEUE.lock().push(old_task);

            // Preparar a nova task
            prepare_and_switch_to(&mut next, Some(old_ctx_ptr), current_guard);
        } else {
            // Se a task antiga JÁ NÃO ESTAVA Running (ex: morreu ou foi colocada em outra fila),
            // apenas trocamos para a próxima sem salvar contexto.
            prepare_and_switch_to(&mut next, None, current_guard);
        }
    } else {
        // Sem task antiga (ex: a anterior dormiu e limpou o CURRENT)
        prepare_and_switch_to(&mut next, None, current_guard);
    }
}

/// Prepara registradores, pilha e CR3 e efetua a troca de contexto
fn prepare_and_switch_to(
    next: &mut Pin<Box<Task>>,
    old_ctx: Option<*mut crate::sched::task::context::CpuContext>,
    mut current_guard: crate::sync::SpinlockGuard<Option<Pin<Box<Task>>>>,
) {
    // 1. Marcar nova task como Running
    unsafe { Pin::get_unchecked_mut(next.as_mut()) }.state = TaskState::Running;

    // 2. Extrair dados necessários
    let next_ref = next.as_ref();
    let next_task = Pin::get_ref(next_ref);
    let new_ctx_ptr = &next_task.context as *const _;
    let new_cr3 = next_task.cr3;
    let stack_top = next_task.kernel_stack.as_u64();
    let is_new = next_task.state == TaskState::Created;

    // 3. Atualizar CURRENT
    // Precisamos fazer o take() do Box para colocar no guard
    // SAFETY: Temos o ownership via parâmetro mut
    let next_box = unsafe { core::ptr::read(next as *mut _ as *const Pin<Box<Task>>) };
    *current_guard = Some(next_box);
    drop(current_guard); // Soltar lock antes da "mágica"

    // 4. Efetuar a troca de hardware
    unsafe {
        // Configurar stack do kernel para interrupções/syscalls
        if stack_top != 0 {
            crate::arch::x86_64::gdt::set_kernel_stack(stack_top);
            crate::arch::x86_64::syscall::set_kernel_rsp(stack_top);
        }

        // Trocar espaço de endereçamento (CR3)
        if new_cr3 != 0 {
            core::arch::asm!("mov cr3, {}", in(reg) new_cr3);
        }

        // Escolher método de restauração
        if let Some(old_ctx_ptr) = old_ctx {
            // Troca clássica: salva e restaura
            crate::sched::task::context::switch(&mut *old_ctx_ptr, &*new_ctx_ptr);
        } else {
            // Troca de "salto": apenas restaura
            // Se é Created (RIP = trampolim), usamos o jump_to_context que limpa stack
            // Se já rodou (RIP = meio do código), usamos o switch com um dummy old
            if is_new {
                crate::sched::task::context::jump_to_context(&*new_ctx_ptr);
            } else {
                // Truque: usamos um CpuContext temporário no stack para o switch ignorar o "salvamento"
                let mut dummy = crate::sched::task::context::CpuContext::new();
                crate::sched::task::context::switch(&mut dummy, &*new_ctx_ptr);
            }
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
