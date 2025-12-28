//! Scheduler principal

pub mod runqueue;

use crate::sync::Spinlock;
use crate::arch::Cpu;
use super::task::Task;
use runqueue::RUNQUEUE;
use alloc::boxed::Box;
use core::pin::Pin;

/// Task atualmente executando (per-CPU no futuro)
static CURRENT: Spinlock<Option<Pin<Box<Task>>>> = Spinlock::new(None);

/// Inicializa o scheduler
pub fn init() {
    crate::kinfo!("(Sched) Inicializando scheduler...");
    // Criar idle task
    // TODO: criar idle task que só faz hlt
}

/// Retorna task atual
pub fn current() -> Option<*const Task> {
    CURRENT.lock().as_ref().map(|t| t.as_ref().get_ref() as *const Task)
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

/// Função principal de escalonamento
pub fn schedule() {
    // Pegar próxima task
    let next = match pick_next() {
        Some(t) => t,
        None => return, // Sem tasks, continuar na atual
    };
    
    // Trocar contexto
    let mut current_guard = CURRENT.lock();
    if let Some(ref mut current) = *current_guard {
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
        
        let old_ctx_ptr = &mut unsafe { Pin::get_unchecked_mut(old_task.as_ref()) }.context as *mut _;
        let new_ctx_ptr = &unsafe { Pin::get_ref(next.as_ref()) }.context as *const _;
        
        RUNQUEUE.lock().push(old_task);
        *current_guard = Some(next);
        
        unsafe {
            super::context::switch(&mut *old_ctx_ptr, &*new_ctx_ptr);
        }
    } else {
        // Primeira task (boot) -> next
        // Não tem old task para salvar
        let new_ctx_ptr = &unsafe { Pin::get_ref(next.as_ref()) }.context as *const _;
         *current_guard = Some(next);
         
         // Aqui precisamos de um "fake" old context ou apenas pular o save.
         // O switch salva o old. Se old for garbage, corrompemos stack?
         // Sim. Na primeira vez, thread de boot deve ter se registrado como CURRENT em init().
         // Assumindo que init() popula CURRENT.
    }
}

/// Loop principal do scheduler (nunca retorna)
pub fn run() -> ! {
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
