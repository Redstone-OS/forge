//! # Orquestrador de Agendamento Multi-Core
//!
//! Lógica central do agendador SMP. Cada CPU tem sua própria runqueue.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    schedule() FLOW                      │
//! ├─────────────────────────────────────────────────────────┤
//! │                                                         │
//! │   1. this_cpu_id()  ──► Obtém ID da CPU atual          │
//! │          │                                              │
//! │          ▼                                              │
//! │   2. CPUS[cpu_id].lock()  ──► Lock IRQ-safe            │
//! │          │                                              │
//! │          ▼                                              │
//! │   3. Verifica estado:                                   │
//! │      ├── current Running + runqueue vazia → return     │
//! │      ├── current Running + runqueue não vazia → swap   │
//! │      └── current Sleeping/Blocked → switch to idle     │
//! │          │                                              │
//! │          ▼                                              │
//! │   4. context_switch()  ──► Troca registradores         │
//! │                                                         │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Invariantes
//!
//! - `schedule()` só toca na runqueue da própria CPU
//! - O Spinlock já desabilita interrupções (IRQ-safe)
//! - Idle task nunca entra na runqueue

use crate::arch::Cpu;
use crate::sched::task::context::CpuContext;
use crate::sched::task::Task;
use crate::sched::task::TaskState;
use alloc::boxed::Box;
use core::pin::Pin;

use super::per_cpu::{this_cpu_id, CpuState, CPUS};

// =============================================================================
// INICIALIZAÇÃO
// =============================================================================

/// Inicializa o subsistema de agendamento
pub fn init() {
    // Inicializa CPU 0 (BSP)
    super::per_cpu::init_cpu(0);
    crate::kinfo!("[SCHED] Sistema de agendamento multi-core pronto.");
}

// =============================================================================
// TICK DO TIMER
// =============================================================================

/// Chamado a cada tick do timer para contabilização
pub fn timer_tick() {
    let cpu_id = this_cpu_id();

    let mut guard = CPUS[cpu_id].lock();
    if let Some(ref mut cpu_data) = *guard {
        if let Some(ref mut task) = cpu_data.current {
            if task.state == TaskState::Running {
                if task.accounting.quantum_left > 0 {
                    task.accounting.quantum_left -= 1;
                }

                if task.accounting.quantum_left == 0 {
                    cpu_data
                        .need_resched
                        .store(true, core::sync::atomic::Ordering::Release);
                }
            }
        }
    }
}

// =============================================================================
// ACESSO À TASK ATUAL
// =============================================================================

/// Retorna ponteiro para a task atual desta CPU
pub fn current() -> Option<*const Task> {
    let cpu_id = this_cpu_id();
    let guard = CPUS[cpu_id].lock();

    if let Some(ref cpu_data) = *guard {
        cpu_data
            .current
            .as_ref()
            .map(|t| t.as_ref().get_ref() as *const Task)
    } else {
        None
    }
}

/// Executa closure com referência imutável à task atual
///
/// Retorna None se não há task atual.
pub fn with_current<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&Task) -> R,
{
    let cpu_id = this_cpu_id();
    let guard = CPUS[cpu_id].lock();

    if let Some(ref cpu_data) = *guard {
        cpu_data
            .current
            .as_ref()
            .map(|task| f(task.as_ref().get_ref()))
    } else {
        None
    }
}

/// Executa closure com referência mutável à task atual
///
/// Retorna None se não há task atual.
pub fn with_current_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut Task) -> R,
{
    let cpu_id = this_cpu_id();
    let mut guard = CPUS[cpu_id].lock();

    if let Some(ref mut cpu_data) = *guard {
        if let Some(ref mut task) = cpu_data.current {
            let task_ref = unsafe { Pin::get_unchecked_mut(task.as_mut()) };
            return Some(f(task_ref));
        }
    }
    None
}

// =============================================================================
// OPERAÇÕES DE FILA
// =============================================================================

/// Adiciona task à runqueue da CPU com menor carga
pub fn enqueue(task: Pin<Box<Task>>) {
    let tid = task.tid.as_u32();

    if tid == 0 || tid >= 0x80000000 {
        crate::kerror!("(Sched) Tentativa de enfileirar idle task! Ignorando.");
        return;
    }

    // Encontrar CPU com menor carga para balanceamento
    let target_cpu = super::per_cpu::LoadBalancer::find_idlest_cpu().unwrap_or(0);
    let current_cpu = this_cpu_id();

    // Enfileirar com lock e capturar resultado
    let enqueued_cpu = {
        let mut guard = CPUS[target_cpu].lock();

        if let Some(ref mut cpu_data) = *guard {
            cpu_data.enqueue(task);
            Some(target_cpu)
        } else {
            drop(guard);
            // Fallback para CPU 0 se a CPU selecionada não estiver inicializada
            let mut guard0 = CPUS[0].lock();
            if let Some(ref mut cpu_data) = *guard0 {
                cpu_data.enqueue(task);
                Some(0)
            } else {
                crate::kerror!("(Sched) Nenhuma CPU inicializada!");
                None
            }
        }
    }; // Lock é liberado aqui ANTES de enviar IPI

    // Se enfileiramos em uma CPU diferente da atual, enviar IPI para acordá-la
    // IMPORTANTE: Lock já foi liberado para evitar deadlock
    if let Some(cpu) = enqueued_cpu {
        if cpu != current_cpu {
            // Obter APIC ID da CPU destino e enviar IPI Reschedule
            if let Some(cpu_info) = crate::core::smp::topology::get().get(cpu) {
                crate::core::smp::ipi::send_reschedule(cpu_info.apic_id);
            }
        }
    }
}

/// Seleciona próxima task para executar (da CPU atual)
pub fn pick_next() -> Option<Pin<Box<Task>>> {
    let cpu_id = this_cpu_id();
    let mut guard = CPUS[cpu_id].lock();

    if let Some(ref mut cpu_data) = *guard {
        cpu_data.dequeue()
    } else {
        None
    }
}

// =============================================================================
// YIELD E SLEEP
// =============================================================================

/// Yield: cede CPU voluntariamente
pub fn yield_now() {
    Cpu::disable_interrupts();
    schedule();
    Cpu::enable_interrupts();
}

/// Sleep: coloca a task atual dormindo por N milissegundos
pub fn sleep_current(ms: u64) {
    if ms == 0 {
        yield_now();
        return;
    }

    Cpu::disable_interrupts();

    let cpu_id = this_cpu_id();
    {
        let mut guard = CPUS[cpu_id].lock();
        if let Some(ref mut cpu_data) = *guard {
            if let Some(ref mut task) = cpu_data.current {
                let now = crate::core::time::jiffies::get_jiffies();
                let ticks = crate::core::time::jiffies::millis_to_jiffies(ms);

                unsafe { Pin::get_unchecked_mut(task.as_mut()) }.wake_at = Some(now + ticks);
                unsafe { Pin::get_unchecked_mut(task.as_mut()) }.state = TaskState::Sleeping;
            }
        }
    }

    schedule();
    Cpu::enable_interrupts();
}

/// Libera lock do scheduler (para novas tasks)
#[no_mangle]
pub unsafe extern "C" fn release_scheduler_lock() {
    let cpu_id = this_cpu_id();
    CPUS[cpu_id].force_unlock();
}

// =============================================================================
// EXIT
// =============================================================================

/// Exit: termina processo atual
pub fn exit_current(code: i32) -> ! {
    Cpu::disable_interrupts();

    let cpu_id = this_cpu_id();
    {
        let mut guard = CPUS[cpu_id].lock();
        if let Some(ref mut cpu_data) = *guard {
            if let Some(mut old_task) = cpu_data.current.take() {
                unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.exit_code = Some(code);
                crate::sched::task::lifecycle::add_zombie(old_task);
            }
        }
    }

    schedule();

    loop {
        schedule();
        Cpu::enable_interrupts();
        Cpu::halt();
        Cpu::disable_interrupts();
    }
}

// =============================================================================
// SCHEDULE PRINCIPAL
// =============================================================================

/// Função principal de escalonamento
///
/// Opera apenas na CPU atual. Cada CPU chama seu próprio schedule().
#[no_mangle]
pub extern "C" fn schedule() {
    let cpu_id = this_cpu_id();

    // Tenta lock (não bloqueia se não conseguir - evita reentrância)
    let mut guard = match CPUS[cpu_id].try_lock() {
        Some(g) => g,
        None => return,
    };

    let cpu_data = match guard.as_mut() {
        Some(data) => data,
        None => return, // CPU não inicializada
    };

    // Pega próxima task da runqueue local
    let mut next_opt = cpu_data.dequeue();

    // Filtra idle task (nunca deve estar na runqueue)
    while let Some(ref task) = next_opt {
        let tid = task.tid.as_u32();
        if tid == 0 || tid >= 0x80000000 {
            crate::kerror!("(Sched) BUG: Idle task na runqueue! Removendo.");
            next_opt = cpu_data.dequeue();
        } else {
            break;
        }
    }

    // CASO A: Não há próxima task
    if next_opt.is_none() {
        if let Some(ref task) = cpu_data.current {
            if task.state == TaskState::Running {
                cpu_data.state = CpuState::Running;
                return;
            }
        }

        // Precisa ir para idle
        if let Some(mut old_task) = cpu_data.current.take() {
            let old_tid = old_task.tid.as_u32();

            // Já é idle - restaura e retorna
            if old_tid == 0 || old_tid >= 0x80000000 {
                unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Running;
                cpu_data.current = Some(old_task);
                cpu_data.state = CpuState::Idle;
                return;
            }

            // Salva task antiga
            let old_ctx_ptr = unsafe {
                &mut Pin::get_unchecked_mut(old_task.as_mut()).context as *mut CpuContext
            };

            match old_task.state {
                TaskState::Sleeping => {
                    super::sleep_queue::add_task(old_task);
                }
                _ => {
                    unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
                    cpu_data.enqueue(old_task);
                }
            }

            // Switch para idle
            cpu_data.state = CpuState::Idle;
            drop(guard);

            if super::idle::is_initialized_for_cpu(cpu_id) {
                unsafe { super::idle::switch_to_idle_cpu(cpu_id, old_ctx_ptr) };
            }
            return;
        }

        cpu_data.state = CpuState::Idle;
        return;
    }

    // CASO B: Há próxima task
    let mut next = next_opt.unwrap();
    let is_new = next.state == TaskState::Created;

    if let Some(mut old_task) = cpu_data.current.take() {
        let old_tid = old_task.tid.as_u32();
        let old_state = old_task.state;
        let is_old_idle = old_tid == 0 || old_tid >= 0x80000000;

        let old_ctx_ptr =
            unsafe { &mut Pin::get_unchecked_mut(old_task.as_mut()).context as *mut CpuContext };

        // Gerencia task antiga
        if old_state == TaskState::Running && !is_old_idle {
            unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
            cpu_data.enqueue(old_task);
        } else if old_state == TaskState::Sleeping {
            super::sleep_queue::add_task(old_task);
        } else if !is_old_idle {
            unsafe { Pin::get_unchecked_mut(old_task.as_mut()) }.state = TaskState::Ready;
            cpu_data.enqueue(old_task);
        }

        // Prepara nova task
        let new_ctx_ptr = &next.context as *const CpuContext;
        unsafe { Pin::get_unchecked_mut(next.as_mut()) }.state = TaskState::Running;
        unsafe { next.apply_hardware_state() };

        cpu_data.current = Some(next);
        cpu_data.state = CpuState::Running;
        drop(guard);

        // Efetua switch
        if is_old_idle {
            let idle_ctx_ptr = unsafe { super::idle::get_idle_context_cpu(cpu_id) };
            unsafe { crate::sched::task::context::switch(&mut *idle_ctx_ptr, &*new_ctx_ptr) };
        } else {
            unsafe { crate::sched::task::context::switch(&mut *old_ctx_ptr, &*new_ctx_ptr) };
        }
    } else {
        // Primeira execução
        let new_ctx_ptr = &next.context as *const CpuContext;
        unsafe { Pin::get_unchecked_mut(next.as_mut()) }.state = TaskState::Running;
        unsafe { next.apply_hardware_state() };

        cpu_data.current = Some(next);
        cpu_data.state = CpuState::Running;
        drop(guard);

        if is_new {
            unsafe { crate::sched::task::context::jump_to_context(&*new_ctx_ptr) };
        } else {
            let mut dummy = CpuContext::new();
            unsafe { crate::sched::task::context::switch(&mut dummy, &*new_ctx_ptr) };
        }
    }
}

// =============================================================================
// RUN LOOP
// =============================================================================

/// Loop principal do scheduler
///
/// Nunca retorna. Cada CPU chama seu próprio run().
pub fn run() -> ! {
    Cpu::disable_interrupts();
    let cpu_id = this_cpu_id();

    crate::kdebug!("(Sched) CPU", cpu_id as u64, "entrando no loop principal");

    loop {
        schedule();
        crate::sched::task::lifecycle::cleanup_all();

        let is_empty = {
            let guard = CPUS[cpu_id].lock();
            guard
                .as_ref()
                .map(|d| d.runqueue_is_empty())
                .unwrap_or(true)
        };

        if is_empty {
            Cpu::enable_interrupts();
            Cpu::halt();
            Cpu::disable_interrupts();
        }
    }
}
