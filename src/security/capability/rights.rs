//! Direitos de capability

/// Direitos que uma capability pode conceder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct CapRights(u32);

impl CapRights {
    pub const NONE: Self = Self(0);
    
    /// Pode ler dados
    pub const READ: Self = Self(1 << 0);
    /// Pode escrever dados
    pub const WRITE: Self = Self(1 << 1);
    /// Pode executar
    pub const EXECUTE: Self = Self(1 << 2);
    /// Pode duplicar o handle
    pub const DUPLICATE: Self = Self(1 << 3);
    /// Pode transferir via IPC
    pub const TRANSFER: Self = Self(1 << 4);
    /// Pode criar capabilities derivadas
    pub const GRANT: Self = Self(1 << 5);
    /// Pode revogar capabilities derivadas
    pub const REVOKE: Self = Self(1 << 6);
    /// Pode esperar em evento
    pub const WAIT: Self = Self(1 << 7);
    /// Pode sinalizar evento
    pub const SIGNAL: Self = Self(1 << 8);
    
    /// Todos os direitos de dados
    pub const RW: Self = Self(Self::READ.0 | Self::WRITE.0);
    /// Todos os direitos
    pub const ALL: Self = Self(0x1FF);
    
    /// Verifica se tem direito específico
    #[inline]
    pub const fn has(self, right: Self) -> bool {
        (self.0 & right.0) == right.0
    }
    
    /// União de direitos
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    
    /// Interseção de direitos
    #[inline]
    pub const fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
    
    /// Remove direitos
    #[inline]
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}
