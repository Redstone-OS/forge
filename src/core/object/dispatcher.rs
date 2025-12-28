use super::kobject::{KObject, Koid};
/// Arquivo: core/object/dispatcher.rs
///
/// Propósito: Despachante de Objetos (Dispatcher).
/// O Dispatcher é o wrapper que envolve um `KObject` e gerencia o acesso a ele via Handles.
/// Ele contém o estado de sinalização (Signals) e observadores (StateObserver).
///
/// Detalhes de Implementação:
/// - Wrappa `Arc<dyn KObject>`.
/// - Handles apontam para Dispatchers (via Arc<Dispatcher>).
/// - Permite que múltiplos handles apontem para o mesmo objeto.
// Objeto Dispatcher
use alloc::sync::Arc;

/// O Dispatcher envolve um KObject e adiciona gerenciamento de estado (sinais).
#[derive(Debug)]
pub struct Dispatcher {
    /// O objeto do kernel real subjacente.
    object: Arc<dyn KObject>,
    // TODO: StateTracker / Signals (user signals, system signals)
    // signals: AtomicU32,
}

impl Dispatcher {
    /// Cria um novo Dispatcher para um dado KObject.
    pub fn new(object: Arc<dyn KObject>) -> Self {
        Self { object }
    }

    /// Retorna referência ao objeto subjacente.
    pub fn object(&self) -> &Arc<dyn KObject> {
        &self.object
    }

    /// Retorna o KOID do objeto.
    pub fn koid(&self) -> Koid {
        self.object.koid()
    }

    // TODO: Métodos de sinalização (signal_set, signal_clear)
}
