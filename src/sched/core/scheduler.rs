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
    if task.tid.as_u32() == 0 {
        crate::kerror!("(Sched) ERRO: Tentativa de colocar PID 0 na RunQueue! Ignorando...");
        return;
    }
    crate::ktrace!("(Sched) enqueue PID:", task.tid.as_u32() as u64);
    RUNQUEUE.lock().push(task);
}

/// Seleciona próxima task para executar
pub fn pick_next() -> Option<Pin<Box<Task>>> {
    let mut rq = RUNQUEUE.lock();
    let res = rq.pop();
    if let Some(ref t) = res {
        crate::ktrace!("(Sched) pick_next: selecionado PID:", t.tid.as_u32() as u64);
    }
    res
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

    // 1. Marcar a task atual como Sleeping e definir tempo
    {
        let mut current_guard = CURRENT.lock();
        if let Some(ref mut task) = *current_guard {
            let now = crate::core::time::jiffies::get_jiffies();
            let ticks = crate::core::time::jiffies::millis_to_jiffies(ms);

            unsafe { Pin::get_unchecked_mut(task.as_mut()) }.wake_at = Some(now + ticks);
            unsafe { Pin::get_unchecked_mut(task.as_mut()) }.state = TaskState::Sleeping;

            crate::kdebug!("(Sched) Task Sleeping...");
        }
    }

    // 2. Chama o schedule.
    // Como a task está Sleeping, o schedule vai salvar o contexto e movê-la para a SleepQueue.
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
    let mut next_opt = pick_next();

    // TODO: Investigar por que tarefas com PID 0 (TID 0) estão entrando na RunQueue.
    // Solução paliativa: Ignora qualquer tarefa com PID 0 para evitar o crash de contexto nulo.
    while let Some(ref task) = next_opt {
        if task.tid.as_u32() == 0 {
            crate::kerror!(
                "(Sched) AVISO: Detectada task com PID 0 inválida na RunQueue! Pulando..."
            );

            dump_tasks();

            next_opt = pick_next();
        } else {
            break;
        }
    }

    let mut current_guard = CURRENT.lock();

    // CASO A: Não há nenhuma task pronta na RunQueue
    if next_opt.is_none() {
        if let Some(mut old_task) = current_guard.take() {
            if old_task.state == TaskState::Running {
                // Task atual pode continuar rodando (ex: yield voluntário sem concorrência)
                *current_guard = Some(old_task);
                return;
            }

            // Task atual suspensa (Sleeping ou Blocked).
            // Precisamos salvar seu contexto e movê-la para a fila correta.
            let state = old_task.state;
            let old_ctx_ptr =
                unsafe { &mut Pin::get_unchecked_mut(old_task.as_mut()).context as *mut _ };

            if state == TaskState::Sleeping {
                crate::ktrace!(
                    "(Sched) Task colocada em SLEEP_QUEUE. PID:",
                    old_task.tid.as_u32() as u64
                );
                super::sleep_queue::add_task(old_task);
            }

            // Agora o sistema está realmente sem tarefas prontas.
            crate::kinfo!("(Sched) Nenhuma task disponível. Entrando em modo Idle.");
            drop(current_guard);

            let mut idle_count = 0u64;
            loop {
                Cpu::enable_interrupts();
                Cpu::halt();
                Cpu::disable_interrupts();

                idle_count += 1;
                if idle_count % 100 == 0 {
                    crate::ktrace!("(Sched) Idler pulsação:", idle_count);
                }

                if let Some(next) = pick_next() {
                    crate::kinfo!("(Sched) Task acordou! Retomando escalonamento.");
                    let g = CURRENT.lock();
                    // Usamos old_ctx_ptr para salvar o estado da task que iniciou o sleep
                    prepare_and_switch_to(next, Some(old_ctx_ptr), g);
                    return; // Jump
                }
            }
        } else {
            // Sistema totalmente ocioso
            crate::kinfo!("(Sched) Sistema ocioso. Aguardando interrupções...");
            drop(current_guard);
            loop {
                Cpu::enable_interrupts();
                Cpu::halt();
                Cpu::disable_interrupts();
                if let Some(next) = pick_next() {
                    let g = CURRENT.lock();
                    prepare_and_switch_to(next, None, g);
                    return;
                }
            }
        }
    }

    // CASO B: Há uma próxima task para rodar (next_opt é Some)
    let next = next_opt.unwrap();
    if let Some(mut old_task) = current_guard.take() {
        let state = old_task.state;

        if state == TaskState::Running || state == TaskState::Sleeping {
            let old_ctx_ptr =
                unsafe { &mut Pin::get_unchecked_mut(old_task.as_mut()).context as *mut _ };

            if state == TaskState::Running {
                unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
                RUNQUEUE.lock().push(old_task);
            } else {
                crate::ktrace!(
                    "(Sched) Task colocada em SLEEP_QUEUE. PID:",
                    old_task.tid.as_u32() as u64
                );
                super::sleep_queue::add_task(old_task);
            }

            prepare_and_switch_to(next, Some(old_ctx_ptr), current_guard);
        } else {
            prepare_and_switch_to(next, None, current_guard);
        }
    } else {
        // Sem task antiga
        prepare_and_switch_to(next, None, current_guard);
    }
}

/// Prepara registradores, pilha e CR3 e efetua a troca de contexto
fn prepare_and_switch_to(
    mut next: Pin<Box<Task>>,
    old_ctx: Option<*mut crate::sched::task::context::CpuContext>,
    mut current_guard: crate::sync::SpinlockGuard<Option<Pin<Box<Task>>>>,
) {
    // 1. Marcar nova task como Running
    unsafe { core::pin::Pin::get_unchecked_mut(next.as_mut()) }.state = TaskState::Running;

    // 2. Extrair dados necessários
    let new_ctx_ptr = &next.context as *const _;
    let new_cr3 = next.cr3;
    let stack_top = next.kernel_stack.as_u64();
    let is_new = next.state == TaskState::Created;

    // [DEBUG - REMOVER DEPOIS] Logar valores do contexto que será carregado para detectar corrupção
    crate::ktrace!(
        "(Sched) Carregando Contexto de PID:",
        next.tid.as_u32() as u64
    );
    crate::ktrace!("  RIP:", next.context.rip);
    crate::ktrace!("  RSP:", next.context.rsp);
    crate::ktrace!("  RBX:", next.context.rbx);
    crate::ktrace!("  RBP:", next.context.rbp);
    crate::ktrace!("  CR3:", new_cr3);

    // 3. Atualizar CURRENT (Dando ownership do Box para o Spinlock)
    *current_guard = Some(next);
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

        // [DEBUG - REMOVER DEPOIS] Log de depuração logo antes do salto final
        crate::kdebug!("(Sched) Efetuando switch/jump final...");

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

/// Imprime o estado de todas as tarefas conhecidas no sistema
pub fn dump_tasks() {
    crate::ktrace!("--- [TRACE] GERENCIADOR DE TAREFAS: LISTA COMPLETA ---");

    // 1. Task Atual (Running)
    if let Some(guard) = CURRENT.try_lock() {
        if let Some(ref task) = *guard {
            crate::ktrace!("  - Running TID:", task.tid.as_u32() as u64);
            crate::ktrace!("    State:", task.state as u32 as u64);
        } else {
            crate::ktrace!("  - CURRENT: None");
        }
    } else {
        crate::ktrace!("  - CURRENT: [Locked]");
    }

    // 2. Ready Tasks
    if let Some(rq) = RUNQUEUE.try_lock() {
        crate::ktrace!("  - READY Tasks count:", rq.queue.len() as u64);
        for task in &rq.queue {
            crate::ktrace!("    -> TID:", task.tid.as_u32() as u64);
        }
    } else {
        crate::ktrace!("  - RUNQUEUE: [Locked]");
    }

    // 3. Sleeping Tasks
    if let Some(sq) = super::sleep_queue::SLEEP_QUEUE.try_lock() {
        crate::ktrace!("  - SLEEPING Tasks count:", sq.len() as u64);
        for task in sq.iter() {
            crate::ktrace!("    -> TID:", task.tid.as_u32() as u64);
        }
    } else {
        crate::ktrace!("  - SLEEP_QUEUE: [Locked]");
    }

    // 4. Zombie Tasks
    if let Some(zombies) = crate::sched::task::lifecycle::ZOMBIES.try_lock() {
        crate::ktrace!("  - ZOMBIE Tasks count:", zombies.len() as u64);
        for task in zombies.iter() {
            crate::ktrace!("    -> TID:", task.tid.as_u32() as u64);
        }
    } else {
        crate::ktrace!("  - ZOMBIES: [Locked]");
    }

    crate::ktrace!("--- [TRACE] FIM DO DUMP ---");
}
