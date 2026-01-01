//! # Orquestrador de Agendamento (High-Level Scheduler)
//!
//! Este arquivo contém a lógica de decisão e gerenciamento de alto nível do agendador.
//! Ele coordena a transição entre estados de tarefas (Running, Sleeping, Ready) e
//! decide quem será o próximo a ocupar a CPU.
//!
//! ## Mecanismos de Execução:
//! - **Cooperativo:** Tarefas cedem voluntariamente via `yield_now()` ou `sleep_current()`.
//! - **Preemptivo:** O sistema retoma o controle via interrupções de hardware (Timer)
//!   quando o quantum da tarefa expira.
//!
//! ## Sincronização:
//! O agendador utiliza um modelo de "Ownership Global" via o Spinlock `CURRENT`.
//! Somente o núcleo que detém o lock da tarefa pode realizar a troca de contexto segura.

use crate::arch::Cpu;
use crate::sched::task::Task;
use crate::sched::task::TaskState;
use crate::sync::Spinlock;
use alloc::boxed::Box;
use core::pin::Pin;

use super::runqueue::RUNQUEUE;

// Task atualmente em execução neste núcleo.
// TODO: Em SMP, este deve ser um campo interno da estrutura `Cpu` ou acessado via GS-base.
pub static CURRENT: Spinlock<Option<Pin<Box<Task>>>> = Spinlock::new(None);

// Inicializa o subsistema de agendamento.
pub fn init() {
    crate::kinfo!("[SCHED] Sistema de agendamento pronto.");
}

// Chamado a cada tick do relógio de hardware para gerenciar o tempo de CPU.
//
// Realiza a contabilização do quantum da tarefa atual e sinaliza se uma
// preempção é necessária.
pub fn timer_tick() {
    // Tentamos o lock. Em interrupções não podemos travar (deadlock) se o kernel já tem o lock.
    if let Some(mut current_guard) = CURRENT.try_lock() {
        if let Some(ref mut task) = *current_guard {
            // Só decrementamos quantum de quem está rodando
            if task.state == TaskState::Running {
                if task.accounting.quantum_left > 0 {
                    task.accounting.quantum_left -= 1;
                }

                // Se o tempo acabou, sinaliza a CPU que precisamos de schedule()
                if task.accounting.quantum_left == 0 {
                    super::cpu::set_need_resched();
                }
            }
        }
    }
}

// Retorna ponteiro para a task atual (unsafe se dereferenciado sem lock, uteil para IDs)
pub fn current() -> Option<*const Task> {
    CURRENT
        .lock()
        .as_ref()
        .map(|t| t.as_ref().get_ref() as *const Task)
}

// Adiciona task à fila de execução
pub fn enqueue(task: Pin<Box<Task>>) {
    if task.tid.as_u32() == 0 {
        crate::kerror!("(Sched) Tentativa de colocar PID 0 na RunQueue! Ignorando...");
        // TODO: Tratar melhor isso
        return;
    }
    crate::ktrace!(
        "(Sched) Nova tarefa na RunQueue PID:",
        task.tid.as_u32() as u64
    );
    RUNQUEUE.lock().push(task);
}

// Seleciona próxima task para executar
pub fn pick_next() -> Option<Pin<Box<Task>>> {
    let mut rq = RUNQUEUE.lock();
    let res = rq.pop();
    if let Some(ref t) = res {
        crate::ktrace!(
            "(Sched) pick_next() selecionado PID:",
            t.tid.as_u32() as u64
        );
    }
    res
}

// Yield: cede CPU voluntariamente
pub fn yield_now() {
    Cpu::disable_interrupts();
    crate::ktrace!("(Sched) yield_now() chamado");
    schedule();
    Cpu::enable_interrupts();
}

// Sleep: coloca a task atual em estado dormente por N milissegundos
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

            crate::kdebug!("(Sched) Tarefa no estado Sleeping");
        }
    }

    // 2. Chama o schedule.
    // Como a task está Sleeping, o schedule vai salvar o contexto e movê-la para a SleepQueue.
    schedule();

    Cpu::enable_interrupts();
}

// Libera o lock do scheduler manualmente (usado por new tasks)
// # Safety
// Somente chamar no início de novas tasks.
#[no_mangle]
pub unsafe extern "C" fn release_scheduler_lock() {
    CURRENT.force_unlock();
}

// Exit: termina processo atual e pula para próximo
pub fn exit_current(code: i32) -> ! {
    Cpu::disable_interrupts();

    // 1. Remover processo atual do CURRENT
    {
        let mut current_guard = CURRENT.lock();
        if let Some(mut old_task) = current_guard.take() {
            // Define o código de saída
            unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.exit_code = Some(code);

            // Move para lista de zumbis para limpeza posterior
            crate::sched::task::lifecycle::add_zombie(old_task);
        }
    }

    // 2. Schedule next
    if let Some(next) = pick_next() {
        let current_guard = CURRENT.lock();
        unsafe {
            super::switch::prepare_and_switch_to(next, None, current_guard);
        }
        // Se chegarmos aqui, algo catastrófico aconteceu, pois a tarefa atual morreu
        // e não deveria mais estar em execução.
        panic!("exit_current: retornou de prepare_and_switch_to!");
    } else {
        // Se a RunQueue está vazia, o sistema entra em modo ocioso (Idle)
        // aguardando novas interrupções que possam acordar processos em sleep.
        unsafe { super::idle::system_idle_loop() };
    }
}

// Função principal de escalonamento
#[no_mangle]
pub extern "C" fn schedule() {
    let mut next_opt = pick_next();

    // Filtro de segurança contra tasks corrompidas (PID 0)
    while let Some(ref task) = next_opt {
        if task.tid.as_u32() == 0 {
            crate::kerror!(
                "(Sched) Detectada task com PID 0 inválida na RunQueue! Pulando para a próxima."
            );
            // TODO: Tratar melhor isso
            super::debug::dump_tasks();
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
            let old_ctx_ptr =
                unsafe { &mut Pin::get_unchecked_mut(old_task.as_mut()).context as *mut _ };
            if old_task.state == TaskState::Sleeping {
                super::sleep_queue::add_task(old_task);
            } else if old_task.state == TaskState::Blocked {
                // Task Blocked sem próxima task: manter na RUNQUEUE como fallback
                crate::kerror!(
                    "(Sched) BUG CASO A: Task Blocked em CURRENT sem próxima task! PID:",
                    old_task.tid.as_u32() as u64
                );
                unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
                RUNQUEUE.lock().push(old_task);
            } else {
                // Estado inesperado: manter na RUNQUEUE como fallback
                crate::kerror!(
                    "(Sched) BUG CASO A: Estado inesperado em CURRENT! PID:",
                    old_task.tid.as_u32() as u64
                );
                crate::kerror!("(Sched) Estado:", old_task.state as u64);
                unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
                RUNQUEUE.lock().push(old_task);
            }

            // Entra no loop de idle (não retorna)
            drop(current_guard);
            unsafe { super::idle::enter_idle_loop(Some(old_ctx_ptr)) };
        } else {
            // Sistema totalmente ocioso (não retorna)
            drop(current_guard);
            unsafe { super::idle::system_idle_loop() };
        }
    }

    // CASO B: Há uma próxima task para rodar
    let next = next_opt.unwrap();
    if let Some(mut old_task) = current_guard.take() {
        let state = old_task.state;

        // DEBUG: Rastrear saída de task
        crate::ktrace!(
            "(Sched) Trocando contexto PID:",
            old_task.tid.as_u32() as u64
        );
        // crate::ktrace!("(Sched) Old State (2=Run,3=Ready,4=Sleep):", state as u64);
        // Nota: TaskState não tem cast direto fácil aqui sem usar unsafe transmute ou Debug,
        // mas vamos assumir que state reflete o valor correto.
        if state == TaskState::Running || state == TaskState::Sleeping {
            let old_ctx_ptr =
                unsafe { &mut Pin::get_unchecked_mut(old_task.as_mut()).context as *mut _ };
            if state == TaskState::Running {
                unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
                RUNQUEUE.lock().push(old_task);
            } else {
                super::sleep_queue::add_task(old_task);
            }
            unsafe { super::switch::prepare_and_switch_to(next, Some(old_ctx_ptr), current_guard) };
        } else if state == TaskState::Blocked {
            // Task Blocked: Algo está errado - tasks Blocked não devem estar em CURRENT
            // pois WaitQueue.wait() as remove de CURRENT antes de bloquear.
            // Isso indica um bug em outro lugar, mas para evitar perda de tasks,
            // fazemos fallback para RUNQUEUE.
            crate::kerror!(
                "(Sched) Task Blocked encontrada em CURRENT! PID:",
                old_task.tid.as_u32() as u64
            );
            crate::kerror!(
                "(Sched) Isso indica bug em IPC/WaitQueue. Movendo para RUNQUEUE como fallback."
            );
            let old_ctx_ptr =
                unsafe { &mut Pin::get_unchecked_mut(old_task.as_mut()).context as *mut _ };
            unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
            RUNQUEUE.lock().push(old_task);
            unsafe { super::switch::prepare_and_switch_to(next, Some(old_ctx_ptr), current_guard) };
        } else {
            // Estado inesperado (Created, Zombie, Dead, Stopped, Ready)
            // Isso indica um BUG sério no sistema - não deveria acontecer
            crate::kerror!("(Sched) BUG CRÍTICO: Task com estado inválido em CURRENT!");
            crate::kerror!("(Sched) PID:", old_task.tid.as_u32() as u64);
            crate::kerror!("(Sched) Estado (enum):", state as u64);
            crate::kerror!("(Sched) Movendo para RUNQUEUE como fallback de segurança");
            let old_ctx_ptr =
                unsafe { &mut Pin::get_unchecked_mut(old_task.as_mut()).context as *mut _ };
            unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
            RUNQUEUE.lock().push(old_task);
            unsafe { super::switch::prepare_and_switch_to(next, Some(old_ctx_ptr), current_guard) };
        }
    } else {
        unsafe { super::switch::prepare_and_switch_to(next, None, current_guard) };
    }
}

// Loop principal do scheduler
pub fn run() -> ! {
    Cpu::disable_interrupts();
    loop {
        schedule();
        if RUNQUEUE.lock().is_empty() {
            crate::sched::task::lifecycle::cleanup_all();
            Cpu::enable_interrupts();
            Cpu::halt();
            Cpu::disable_interrupts();
        }
    }
}
