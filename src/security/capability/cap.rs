//! Capability - token de acesso

use super::rights::CapRights;

/// Tipo de objeto que a capability referencia
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CapType {
    /// Slot vazio
    Null = 0,
    /// Memória virtual (VMO)
    Memory = 1,
    /// Porta IPC
    Port = 2,
    /// Canal IPC
    Channel = 3,
    /// Thread
    Thread = 4,
    /// Processo
    Process = 5,
    /// Evento
    Event = 6,
    /// Timer
    Timer = 7,
    /// IRQ
    Irq = 8,
    /// Região MMIO
    Mmio = 9,
    /// Container de capabilities (CNode)
    CNode = 10,
}

/// Uma capability é um token unforgeable de acesso
#[derive(Debug, Clone)]
pub struct Capability {
    /// Tipo do objeto referenciado
    pub cap_type: CapType,
    /// Direitos concedidos
    pub rights: CapRights,
    /// Referência ao objeto (índice interno)
    pub object_ref: u64,
    /// Badge para identificação em IPC
    pub badge: u64,
    /// Generation counter (para revocação)
    pub generation: u32,
}

impl Capability {
    /// Cria capability nula
    pub const fn null() -> Self {
        Self {
            cap_type: CapType::Null,
            rights: CapRights::NONE,
            object_ref: 0,
            badge: 0,
            generation: 0,
        }
    }
    
    /// Cria nova capability
    pub const fn new(cap_type: CapType, rights: CapRights, object_ref: u64) -> Self {
        Self {
            cap_type,
            rights,
            object_ref,
            badge: 0,
            generation: 0,
        }
    }
    
    /// Verifica se é válida
    pub const fn is_valid(&self) -> bool {
        !matches!(self.cap_type, CapType::Null)
    }
    
    /// Cria capability derivada com menos direitos
    pub fn derive(&self, new_rights: CapRights) -> Option<Self> {
        // Só pode reduzir direitos
        if !self.rights.has(CapRights::GRANT) {
            return None;
        }
        
        // Nova capability tem apenas direitos que origem tinha
        let reduced_rights = new_rights.intersect(self.rights);
        
        Some(Self {
            cap_type: self.cap_type,
            rights: reduced_rights.without(CapRights::GRANT),
            object_ref: self.object_ref,
            badge: self.badge,
            generation: self.generation,
        })
    }
}

/// Handle opaco para userspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CapHandle(u32);

impl CapHandle {
    pub const INVALID: Self = Self(0);
    
    pub const fn new(index: u32) -> Self {
        Self(index)
    }
    
    pub const fn as_u32(self) -> u32 {
        self.0
    }
    
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}
