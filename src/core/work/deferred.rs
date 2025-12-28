/// Arquivo: core/work/deferred.rs
///
/// Propósito: Abstração de alto nível para Execução Diferida.
/// Simplifica o uso de WorkQueues para casos comuns de "fire and forget".
///
/// Detalhes de Implementação:
/// - Wrapper conveniente sobre `WorkQueue`.
/// - Permite agendar closures para rodar em contexto de thread (seguro para dormir/bloquear).
// Execução Adiada (Deferred Execution)mples
use crate::core::work::workqueue::{ClosureWork, SYSTEM_WQ};
use alloc::boxed::Box;

/// Agenda uma função para execução futura na fila de trabalho do sistema.
///
/// Este é o método preferido para rodar tarefas de manutenção, limpeza ou
/// continuação de I/O que não requerem latência de tempo real.
///
/// # Exemplo
///
/// ```ignore
/// deferred::defer(|| {
///     kinfo!("Executando em background...");
/// });
/// ```
pub fn defer<F>(f: F)
where
    F: FnMut() + Send + Sync + 'static,
{
    SYSTEM_WQ.enqueue(ClosureWork::new(f));
}

/// Estrutura para manter um trabalho diferido que pode ser reagendado.
pub struct DeferredTask {
    func: Option<Box<dyn FnMut() + Send + Sync>>,
}

impl DeferredTask {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut() + Send + Sync + 'static,
    {
        Self {
            func: Some(Box::new(f)),
        }
    }

    /// Agenda a tarefa. Note que isso consome a função (one-shot por enquanto).
    /// Se precisar rodar múltiplas vezes, a closure deve se clonar ou usar Arc/Mutex internos,
    /// ou precisaríamos mudar WorkItem para não consumir self (mas Box<dyn trait> geralmente consome).
    pub fn schedule(&mut self) {
        if let Some(func) = self.func.take() {
            // Re-envelopar em ClosureWork.
            // Nota: ClosureWork::new espera FnMut.
            // Aqui estamos movendo o Box para dentro do ClosureWork.
            // Para suportar múltiplas execuções, o design de WorkItem precisaria ser ajustado ou
            // usaríamos Arc<Mutex<_>> no estado compartilhado.
            //
            // Como esta é uma implementação simples "fire-once", movemos a função.

            // Hack para converter Box<dyn FnMut> que temos para a estrutura esperada
            // ou simplesmente criamos uma closure que chama o box.
            let mut boxed_fn = func;
            SYSTEM_WQ.enqueue(ClosureWork::new(move || {
                (boxed_fn)();
            }));
        }
    }
}
