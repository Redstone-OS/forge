//! Handle opaco para userspace

/// Handle opaco que representa um objeto do kernel.
///
/// Userspace nunca vê ponteiros reais, apenas handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Handle(u32);

impl Handle {
    /// Handle inválido/nulo
    pub const INVALID: Handle = Handle(0);
    
    /// Cria novo handle a partir de índice
    pub const fn new(index: u32) -> Self {
        Self(index)
    }
    
    /// Retorna o valor raw do handle
    pub const fn raw(&self) -> u32 {
        self.0
    }
    
    /// Verifica se é válido
    pub const fn is_valid(&self) -> bool {
        self.0 != 0
    }
}
