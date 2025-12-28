//! Direitos de acesso a objetos

/// Direitos que um handle pode ter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Rights(u32);

impl Rights {
    /// Sem direitos
    pub const NONE: Rights = Rights(0);
    
    /// Pode ler
    pub const READ: Rights = Rights(1 << 0);
    
    /// Pode escrever
    pub const WRITE: Rights = Rights(1 << 1);
    
    /// Pode executar
    pub const EXECUTE: Rights = Rights(1 << 2);
    
    /// Pode duplicar o handle
    pub const DUPLICATE: Rights = Rights(1 << 3);
    
    /// Pode transferir via IPC
    pub const TRANSFER: Rights = Rights(1 << 4);
    
    /// Pode criar handles derivados
    pub const GRANT: Rights = Rights(1 << 5);
    
    /// Todos os direitos
    pub const ALL: Rights = Rights(0x3F);
    
    /// Verifica se tem direito específico
    pub const fn has(&self, right: Rights) -> bool {
        (self.0 & right.0) == right.0
    }
    
    /// Combina direitos
    pub const fn union(self, other: Rights) -> Rights {
        Rights(self.0 | other.0)
    }
    
    /// Interseção de direitos
    pub const fn intersect(self, other: Rights) -> Rights {
        Rights(self.0 & other.0)
    }
}
