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

/// Inicializa o subsistema de agendamento.
pub fn init() {
    crate::kinfo!("[SCHED] Sistema de agendamento pronto.");
}

/// Chamado a cada tick do relógio de hardware para gerenciar o tempo de CPU.
///
/// Realiza a contabilização do quantum da tarefa atual e sinaliza se uma
/// preempção é necessária.
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

/// Seleciona próxima task para executar
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

/// Yield: cede CPU voluntariamente
pub fn yield_now() {
    Cpu::disable_interrupts();
    crate::ktrace!("(Sched) yield_now() chamado");
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

            crate::kdebug!("(Sched) Tarefa no estado Sleeping");
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

    // 2. Schedule next (ou idle task se não houver mais nada)
    // schedule() retorna para a idle task se não houver próxima task
    schedule();

    // Se chegarmos aqui após exit, continue no loop do scheduler
    loop {
        schedule();
        Cpu::enable_interrupts();
        Cpu::halt();
        Cpu::disable_interrupts();
    }
}

/// Função principal de escalonamento
#[no_mangle]
pub extern "C" fn schedule() {
    let mut next_opt = pick_next();

    // Filtro de segurança contra tasks corrompidas (PID 0 não deve estar na RunQueue)
    while let Some(ref task) = next_opt {
        if task.tid.as_u32() == 0 {
            crate::kerror!("(Sched) BUG: Idle task (PID 0) encontrada na RunQueue! Removendo.");
            next_opt = pick_next();
        } else {
            break;
        }
    }

    let mut current_guard = CURRENT.lock();

    // CASO A: Não há nenhuma task pronta na RunQueue
    if next_opt.is_none() {
        if let Some(ref old_task) = *current_guard {
            // Se a task atual pode continuar rodando, deixa ela
            if old_task.state == TaskState::Running {
                // Idle task ou qualquer task Running pode continuar
                return;
            }

            // Task atual suspensa (Sleeping ou Blocked) - precisamos tirar de CURRENT
            // mas só podemos fazer isso se tivermos para onde ir (idle task)
            let is_idle = old_task.tid.as_u32() == 0;

            if is_idle {
                // Idle task em estado não-Running sem próxima task é um bug
                crate::kerror!("(Sched) BUG: Idle task não está Running mas não há próxima task!");
                // Força Running e continua
                unsafe {
                    let task_ptr = current_guard.as_mut().unwrap();
                    Pin::get_unchecked_mut(task_ptr.as_mut()).state = TaskState::Running;
                }
                return;
            }

            // Task normal suspensa - precisamos fazer switch para algum lugar
            // Mas não temos idle task disponível ainda, então re-enfileira e retorna
            // NOTA: Isso só deveria acontecer antes da idle task ser inicializada
            let mut old_task = current_guard.take().unwrap();

            if old_task.state == TaskState::Sleeping {
                super::sleep_queue::add_task(old_task);
            } else {
                // Blocked ou outro estado - re-enfileira com aviso
                crate::kwarn!("(Sched) Task não-Running sem próxima: movendo para RunQueue");
                unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
                RUNQUEUE.lock().push(old_task);
            }

            // Sem CURRENT e sem próxima task - situação crítica
            drop(current_guard);
            crate::kerror!("(Sched) CURRENT vazio e sem próxima task! Sistema pode travar.");
            return;
        }
        // Nenhuma task em CURRENT e nenhuma na RunQueue
        // Isso é esperado logo após init, antes da idle task ser criada
        return;
    }

    // CASO B: Há uma próxima task para rodar
    let next = next_opt.unwrap();
    let next_pid = next.tid.as_u32();

    if let Some(mut old_task) = current_guard.take() {
        let old_pid = old_task.tid.as_u32();
        let state = old_task.state;
        let is_old_idle = old_pid == 0;

        crate::ktrace!("(Sched) Trocando contexto PID:", old_pid as u64);

        // Obtém ponteiro para contexto da task antiga
        let old_ctx_ptr =
            unsafe { &mut Pin::get_unchecked_mut(old_task.as_mut()).context as *mut _ };

        // Gerencia a task antiga baseado em seu estado
        if state == TaskState::Running {
            if is_old_idle {
                // Idle task cedendo CPU - guarda na CURRENT temporariamente
                // Vai ser substituída pelo prepare_and_switch_to
                // Não colocamos idle na RunQueue!
            } else {
                unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
                RUNQUEUE.lock().push(old_task);
            }
        } else if state == TaskState::Sleeping {
            super::sleep_queue::add_task(old_task);
        } else if state == TaskState::Blocked {
            // Task Blocked deveria ter sido removida de CURRENT por WaitQueue
            crate::kerror!("(Sched) Task Blocked em CURRENT! PID:", old_pid as u64);
            unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
            RUNQUEUE.lock().push(old_task);
        } else {
            // Estado inesperado
            crate::kerror!("(Sched) Estado inesperado em CURRENT! PID:", old_pid as u64);
            crate::kerror!("(Sched) Estado:", state as u64);
            if !is_old_idle {
                unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
                RUNQUEUE.lock().push(old_task);
            }
        }

        // Para idle task cedendo CPU, não salvamos nada especial
        // O contexto é salvo normalmente em old_ctx_ptr
        if is_old_idle {
            // Não coloca idle na RunQueue, mas ainda precisamos do contexto
            unsafe { super::switch::prepare_and_switch_to(next, Some(old_ctx_ptr), current_guard) };
        } else {
            unsafe { super::switch::prepare_and_switch_to(next, Some(old_ctx_ptr), current_guard) };
        }
    } else {
        // Nenhuma task atual - primeira execução ou após idle
        crate::ktrace!("(Sched) Primeira execução de PID:", next_pid as u64);
        unsafe { super::switch::prepare_and_switch_to(next, None, current_guard) };
    }
}

/// Loop principal do scheduler
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
