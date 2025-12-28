/// Arquivo: core/object/rights.rs
///
/// Propósito: Definição de Direitos (Rights) sobre Objetos do Kernel.
/// O sistema de segurança é baseado em Capabilities. Cada Handle (referência a um objeto)
/// possui um conjunto de direitos associados que determinam o que pode ser feito.
///
/// Detalhes de Implementação:
/// - Bitmask de 32 bits.
/// - Direitos genéricos (Read, Write, etc.) e específicos podem ser misturados.
/// - Implementação manual de bitflags para evitar dependência externa.

// Direitos de Acesso (Capabilities)

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Rights(pub u32);

impl Rights {
    // --- Direitos Básicos ---
    pub const NONE: Rights = Rights(0);
    pub const DUPLICATE: Rights = Rights(1 << 0);
    pub const TRANSFER: Rights = Rights(1 << 1);
    pub const READ: Rights = Rights(1 << 2);
    pub const WRITE: Rights = Rights(1 << 3);
    pub const EXECUTE: Rights = Rights(1 << 4);
    pub const MAP: Rights = Rights(1 << 5);
    pub const GET_PROPERTY: Rights = Rights(1 << 6);
    pub const SET_PROPERTY: Rights = Rights(1 << 7);

    // --- Direitos Específicos de Tarefas/Processos ---
    pub const ENUMERATE: Rights = Rights(1 << 8);
    pub const DESTROY: Rights = Rights(1 << 9);

    // --- Todos os direitos (Root) ---
    pub const ALL: Rights = Rights(0xFFFFFFFF);

    /// Cria um novo conjunto de direitos vazio
    pub const fn empty() -> Self {
        Self::NONE
    }

    /// Verifica se possui todos os direitos especificados em `other`
    pub const fn contains(&self, other: Rights) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Adiciona direitos
    pub const fn union(&self, other: Rights) -> Self {
        Rights(self.0 | other.0)
    }

    /// Remove direitos (interseção)
    pub const fn intersection(&self, other: Rights) -> Self {
        Rights(self.0 & other.0)
    }
}

// Implementação de Bitwise Operators para facilidade de uso
impl core::ops::BitOr for Rights {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl core::ops::BitAnd for Rights {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}
