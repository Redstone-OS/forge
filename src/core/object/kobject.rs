//! Trait base para objetos do kernel

use super::Rights;

/// Tipos de objetos do kernel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ObjectType {
    None = 0,
    Process = 1,
    Thread = 2,
    Vmo = 3,        // Virtual Memory Object
    Port = 4,       // IPC Port
    Channel = 5,    // IPC Channel
    Event = 6,
    Timer = 7,
    Interrupt = 8,
    Pager = 9,
}

/// Trait que todo objeto do kernel deve implementar
pub trait KernelObject: Send + Sync {
    /// Tipo do objeto
    fn object_type(&self) -> ObjectType;
    
    /// Direitos padrão para este tipo de objeto
    fn default_rights(&self) -> Rights;
    
    /// Chamado quando última referência é liberada
    fn on_destroy(&mut self) {}
}
