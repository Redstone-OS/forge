//! # Revocação de Capabilities
//!
//! Sistema para revogar capabilities derivadas.
//!
//! Quando uma capability é derivada (via GRANT), o original
//! pode revogar todas as derivadas a qualquer momento.

use super::CapHandle;
use alloc::vec::Vec;

// TODO: Revisar no futuro
#[allow(unused)]
/// Nó na árvore de derivação.
struct DerivationNode {
    handle: CapHandle,
    children: Vec<CapHandle>,
}

/// Árvore de derivação para revocação.
///
/// Rastreia quais capabilities foram derivadas de quais.
pub struct RevocationTree {
    /// Mapeamento de handles para nós.
    nodes: Vec<Option<DerivationNode>>,
}

impl RevocationTree {
    /// Capacidade inicial.
    const INITIAL_CAPACITY: usize = 256;

    /// Cria nova árvore.
    pub fn new() -> Self {
        Self {
            nodes: Vec::with_capacity(Self::INITIAL_CAPACITY),
        }
    }

    /// Registra uma capability raiz (sem pai).
    pub fn add_root(&mut self, handle: CapHandle) {
        self.ensure_capacity(handle);

        let index = handle.as_u32() as usize;
        self.nodes[index] = Some(DerivationNode {
            handle,
            children: Vec::new(),
        });
    }

    /// Registra derivação (parent -> child).
    pub fn add_derivation(&mut self, parent: CapHandle, child: CapHandle) {
        self.ensure_capacity(parent);
        self.ensure_capacity(child);

        let parent_idx = parent.as_u32() as usize;
        let child_idx = child.as_u32() as usize;

        // Adicionar filho ao pai
        if let Some(ref mut node) = self.nodes[parent_idx] {
            node.children.push(child);
        }

        // Criar nó para filho
        self.nodes[child_idx] = Some(DerivationNode {
            handle: child,
            children: Vec::new(),
        });
    }

    /// Revoga capability e todos os derivados.
    ///
    /// Retorna lista de handles revogados.
    pub fn revoke(&mut self, handle: CapHandle) -> Vec<CapHandle> {
        let mut revoked = Vec::new();
        self.revoke_recursive(handle, &mut revoked);
        revoked
    }

    /// Revocação recursiva.
    fn revoke_recursive(&mut self, handle: CapHandle, revoked: &mut Vec<CapHandle>) {
        let index = handle.as_u32() as usize;

        if index >= self.nodes.len() {
            return;
        }

        // Pegar nó e remover da árvore
        let node = match self.nodes[index].take() {
            Some(n) => n,
            None => return,
        };

        // Adicionar à lista de revogados
        revoked.push(handle);

        // Revogar filhos recursivamente
        for child in node.children {
            self.revoke_recursive(child, revoked);
        }
    }

    /// Remove handle da árvore (sem revogar filhos).
    pub fn remove(&mut self, handle: CapHandle) {
        let index = handle.as_u32() as usize;
        if index < self.nodes.len() {
            self.nodes[index] = None;
        }
    }

    /// Verifica se handle existe na árvore.
    pub fn contains(&self, handle: CapHandle) -> bool {
        let index = handle.as_u32() as usize;
        index < self.nodes.len() && self.nodes[index].is_some()
    }

    /// Garante capacidade para o índice.
    fn ensure_capacity(&mut self, handle: CapHandle) {
        let index = handle.as_u32() as usize;
        while self.nodes.len() <= index {
            self.nodes.push(None);
        }
    }
}

impl Default for RevocationTree {
    fn default() -> Self {
        Self::new()
    }
}
