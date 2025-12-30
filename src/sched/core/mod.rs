//! Scheduler principal

pub mod cpu;
pub mod policy;
pub mod runqueue;

pub use policy::SchedulingPolicy;

use super::task::Task;
use crate::arch::Cpu;
use crate::sync::Spinlock;
use alloc::boxed::Box;
use core::pin::Pin;
use runqueue::RUNQUEUE;

pub mod entry;

/// Task atualmente executando (per-CPU no futuro)
static CURRENT: Spinlock<Option<Pin<Box<Task>>>> = Spinlock::new(None);

/// Inicializa o scheduler
pub fn init() {
    crate::kinfo!("(Sched) Inicializando scheduler...");

    // A idle task não precisa ser criada explicitamente.
    // O scheduler simplesmente faz hlt quando a runqueue está vazia (em run()).
    // Isso é mais simples e evita overhead de context switch.
    crate::kinfo!("(Sched) Scheduler inicializado. Idle loop integrado.");
}

/// Retorna task atual
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

/// Exit: termina processo atual e pula para próximo
///
/// Diferente de yield, NÃO coloca o processo atual de volta na fila.
/// Nunca retorna.
pub fn exit_current() -> ! {
    Cpu::disable_interrupts();

    // Remover processo atual do CURRENT (ele não vai para a fila de volta)
    {
        let mut current_guard = CURRENT.lock();
        // IMPORTANTE: Não podemos dropar a task ainda porque estamos usando sua stack!
        // Devemos movê-la para a lista de zombies (ou similar) para que outra task limpe.
        if let Some(old_task) = current_guard.take() {
            crate::sched::task::lifecycle::add_zombie(old_task);
        }
    }

    // Pegar próxima task
    if let Some(next) = pick_next() {
        let mut current_guard = CURRENT.lock();

        // Obter referência ao contexto antes de mover
        let next_ref = next.as_ref();
        let ctx_ptr = &{ core::pin::Pin::get_ref(next_ref) }.context
            as *const crate::sched::task::context::CpuContext;
        let new_cr3 = { core::pin::Pin::get_ref(next_ref) }.cr3;
        let kernel_stack = { core::pin::Pin::get_ref(next_ref) }.kernel_stack.as_u64();

        *current_guard = Some(next);
        drop(current_guard);

        unsafe {
            // Configurar stack e GS
            if kernel_stack != 0 {
                crate::arch::x86_64::gdt::set_kernel_stack(kernel_stack);
                crate::arch::x86_64::syscall::set_kernel_rsp(kernel_stack);
            }

            // Trocar CR3
            if new_cr3 != 0 {
                core::arch::asm!("mov cr3, {}", in(reg) new_cr3);
            }

            // Pular para próxima task
            crate::sched::task::context::jump_to_context(&*ctx_ptr);
        }
    } else {
        // Sem tasks, halt infinito
        loop {
            Cpu::enable_interrupts();
            Cpu::halt();
            Cpu::disable_interrupts();
        }
    }
}

/// Função principal de escalonamento
pub fn schedule() {
    // Pegar próxima task
    let next = match pick_next() {
        Some(t) => t,
        None => return, // Sem tasks, continuar na atual
    };

    // Trocar contexto
    let mut current_guard = CURRENT.lock();
    if let Some(ref mut _current) = *current_guard {
        // Salvar task atual de volta na fila
        // Nota: para fazer isso com segurança precisa tomar cuidado com ownership.
        // O guia simplifica, mas vamos seguir a lógica de mover de volta.
        // O `take()` move a task para fora do CURRENT.
        let old_task = current_guard.take().unwrap();

        // Antes de mover, precisamos salvar o contexto atual.
        // Como old_task é Pin<Box<Task>>, podemos acessar o contexto.
        // A lógica de switch vai exigir ponteiros mutáveis.

        // Armazenamos a next em CURRENT antes do switch?
        // Não, switch precisa de ambos.

        // Hack para extrair mut pointers:
        // old_ctx é &mut user::Task::context
        // new_ctx é & user::Task::context

        // Como o guia diz:
        /*
          RUNQUEUE.lock().push(old_task);
          *current_guard = Some(next);
          // contexto....
        */

        // Isso é tricky em Rust seguro. O switch precisa acontecer DEPOIS de atualizar as estruturas,
        // mas precisa das referências das estruturas.

        // Vamos apenas implementar o fluxo lógico por enquanto, ciente que o compilador vai reclamar de moves.
        // No mundo real, usaríamos raw pointers para o switch_asm.

        // SAFETY: Pin<Box<Task>> é obtido de CURRENT. Temos ownership.
        // Precisamos extrair o ponteiro para o contexto.
        // Task é !Unpin (provavelmente), mas temos acesso exclusivo aqui.
        let mut old_task_pin = old_task;
        let old_ctx_ptr =
            &mut unsafe { Pin::get_unchecked_mut(old_task_pin.as_mut()) }.context as *mut _;
        let new_ctx_ptr = &{ Pin::get_ref(next.as_ref()) }.context as *const _;

        RUNQUEUE.lock().push(old_task_pin);
        *current_guard = Some(next);

        unsafe {
            // Atualizar TSS RSP0 para a nova task (para interrupções Ring 3 -> Ring 0)
            // E KERNEL_GS_BASE para syscalls
            if let Some(current_task) = current_guard.as_ref() {
                let stack_top = current_task.as_ref().kernel_stack.as_u64();
                if stack_top != 0 {
                    crate::arch::x86_64::gdt::set_kernel_stack(stack_top);
                    crate::arch::x86_64::syscall::set_kernel_rsp(stack_top);
                }
            }

            crate::sched::task::context::switch(&mut *old_ctx_ptr, &*new_ctx_ptr);
        }
    } else {
        // Primeira task (boot) -> next
        // Não tem old task para salvar, usamos jump_to_context
        crate::ktrace!("(Sched) Primeira task, usando jump_to_context");

        // Obter referência ao contexto ANTES de mover next para CURRENT
        // Precisamos do CR3 também
        let next_ref = next.as_ref();
        let ctx_ptr =
            &{ Pin::get_ref(next_ref) }.context as *const crate::sched::task::context::CpuContext;
        let new_cr3 = { Pin::get_ref(next_ref) }.cr3;
        let kernel_stack = { Pin::get_ref(next_ref) }.kernel_stack.as_u64();

        *current_guard = Some(next);

        // Liberar o guard antes do jump (não vai retornar)
        drop(current_guard);

        // Logar RIP targets
        unsafe {
            crate::ktrace!("(Sched) Saltando para contexto. RIP=", (*ctx_ptr).rip);
            crate::ktrace!("(Sched) Contexto PTR=", ctx_ptr as u64);
            crate::ktrace!("(Sched) Contexto RSP=", (*ctx_ptr).rsp);
            crate::ktrace!("(Sched) Carregando CR3=", new_cr3);

            // Configurar TSS RSP0 e KERNEL_GS_BASE para syscalls
            if kernel_stack != 0 {
                // crate::ktrace!("(Sched) Configurando kernel_stack=", kernel_stack);
                crate::arch::x86_64::gdt::set_kernel_stack(kernel_stack);
                crate::arch::x86_64::syscall::set_kernel_rsp(kernel_stack);
            } else {
                crate::kerror!("(Sched) AVISO: kernel_stack é 0!");
            }

            if new_cr3 != 0 {
                core::arch::asm!("mov cr3, {}", in(reg) new_cr3);
            }

            let cpu_context_ptr = ctx_ptr as *const u64;
            crate::ktrace!("(Sched) Post-switch checks done.");
            crate::ktrace!("(Sched) POST CpuContext.rsp=", *(cpu_context_ptr.offset(6)));
            crate::ktrace!("(Sched) POST CpuContext.rip=", *(cpu_context_ptr.offset(7)));
            crate::ktrace!("(Sched) POST CpuContext.rip=", (*ctx_ptr).rip);

            crate::sched::task::context::jump_to_context(&*ctx_ptr);
        }
    }
}

/// Loop principal do scheduler (nunca retorna)
pub fn run() -> ! {
    // Garante interrupções desabilitadas ao entrar no loop de scheduling
    Cpu::disable_interrupts();

    loop {
        schedule();

        // Se não há tasks, esperar interrupção
        if RUNQUEUE.lock().is_empty() {
            Cpu::enable_interrupts();
            Cpu::halt();
            Cpu::disable_interrupts();
        }
    }
}

/// Libera o lock do scheduler manualmente (usado por new tasks)
/// # Safety
/// Somente chamar no início de novas tasks.
#[no_mangle]
pub unsafe extern "C" fn release_scheduler_lock() {
    CURRENT.force_unlock();
}
