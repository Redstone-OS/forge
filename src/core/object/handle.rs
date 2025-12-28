//! Handle para objetos do kernel (Capability)
//!
//! Handles são referências seguras e opacas para objetos do kernel,
//! pertencentes a um processo específico.
//!
//! Detalhes de Implementação:
//! - `HandleValue`: O inteiro (u32) visto pelo userspace.
//! - `Handle`: A estrutura interna que associa um Dispatcher a Direitos.

use super::dispatcher::Dispatcher;
use super::rights::Rights;
use alloc::sync::Arc; // Será implementado a seguir

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
        Self { dispatcher, rights }
    }

    // TODO: Métodos para verificar direitos, duplicar com menos direitos, etc.
}
