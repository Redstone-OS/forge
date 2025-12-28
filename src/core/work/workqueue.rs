/// Arquivo: core/work/workqueue.rs
///
/// Propósito: Implementação de Filas de Trabalho (Work Queues).
/// Permite agendar a execução de funções para um momento posterior, fora do contexto de interrupção.
///
/// Detalhes de Implementação:
/// - Usa `VecDeque` protegido por `Spinlock` para armazenar trabalhos.
/// - Suporta execução de itens enfileirados.
/// - Projetado para ser consumido por threads dedicadas (worker threads) no scheduler.

//! Filas de trabalho diferido

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use crate::sync::spinlock::Spinlock;

/// Trait para itens de trabalho
pub trait WorkItem: Send + Sync {
    /// Executa o trabalho
    fn run(&mut self);
}

/// Um item de trabalho genérico (Closure)
pub struct ClosureWork {
    func: Box<dyn FnMut() + Send + Sync>,
}

impl ClosureWork {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() + Send + Sync + 'static,
    {
        Self {
            func: Box::new(f),
        }
    }
}

impl WorkItem for ClosureWork {
    fn run(&mut self) {
        (self.func)();
    }
}

/// Fila de trabalho
pub struct WorkQueue {
    queue: Spinlock<VecDeque<Box<dyn WorkItem>>>,
}

impl WorkQueue {
    /// Cria uma nova WorkQueue
    /// 
    /// Nota: Requer que `VecDeque::new` e `Spinlock::new` sejam const para uso em estáticas.
    pub const fn new() -> Self {
        Self {
            queue: Spinlock::new(VecDeque::new()),
        }
    }

    /// Enfileira um trabalho para execução futura
    pub fn enqueue<W: WorkItem + 'static>(&self, work: W) {
        let mut q = self.queue.lock();
        q.push_back(Box::new(work));
        
        // TODO: Acordar worker thread associada a esta fila se estiver dormindo
        // Isso requer integração com o scheduler (que não podemos importar diretamente aqui)
        // Possível solução: Callback hook ou polling.
    }
    
    /// Processa todos os itens pendentes na fila (Flush)
    pub fn process_all(&self) {
        loop {
            // Retirar um item protegendo o lock o mínimo possível
            let item = {
                let mut q = self.queue.lock();
                q.pop_front()
            };

            match item {
                Some(mut work) => work.run(),
                None => break, // Fila vazia
            }
        }
    }
}

// Fila global de sistema
pub static SYSTEM_WQ: WorkQueue = WorkQueue::new();
