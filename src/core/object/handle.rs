/// Arquivo: core/object/handle.rs
///
/// Propósito: Definição de Handles.
/// Um Handle é uma referência "segura" e "com direitos restritos" a um objeto do kernel,
/// pertencente a um processo específico.
///
/// Detalhes de Implementação:
/// - `HandleValue`: O inteiro (u32) visto pelo userspace.
/// - `Handle`: A estrutura interna que associa um Dispatcher a Direitos.

//! Handles e Tabela de Handles

use alloc::sync::Arc;
use super::rights::Rights;
use super::dispatcher::Dispatcher; // Será implementado a seguir

/// Valor numérico do handle visto pelo usuário (Userspace)
pub type HandleValue = u32;

/// Valor inválido de handle
pub const INVALID_HANDLE: HandleValue = 0;

/// Estrutura interna de um Handle.
/// Mantém a referência contada ao objeto (via Dispatcher) e os direitos de acesso.
#[derive(Debug, Clone)]
pub struct Handle {
    /// O objeto subjacente referenciado.
    pub dispatcher: Arc<Dispatcher>,
    
    /// Os direitos que este handle possui sobre o objeto.
    pub rights: Rights,
}

impl Handle {
    /// Cria um novo handle
    pub fn new(dispatcher: Arc<Dispatcher>, rights: Rights) -> Self {
        Self {
            dispatcher,
            rights,
        }
    }

    // TODO: Métodos para verificar direitos, duplicar com menos direitos, etc.
}
