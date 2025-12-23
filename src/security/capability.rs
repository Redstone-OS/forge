//! Capability-based Security Primitives.
//!
//! No Redstone, segurança não é uma lista de verificação (ACL), é a posse de um token.
//! Uma `Capability` é uma referência opaca a um objeto do kernel com direitos específicos.

use bitflags::bitflags;

/// Tipos de objetos que podem ser referenciados por uma Capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapType {
    /// Capability nula/inválida.
    Null,
    /// Porta de IPC (envio/recebimento).
    Port,
    /// Região de memória física ou virtual.
    Memory,
    /// Interrupção de Hardware.
    Irq,
    /// Dispositivo de IO (Portas IO ou MMIO).
    Device,
    /// Controle de Processo/Tarefa.
    Task,
}

bitflags! {
    /// Direitos de acesso associados a uma Capability.
    /// Define O QUE você pode fazer com o objeto.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CapRights: u32 {
        /// Permite ler do objeto (ex: recv na porta).
        const READ    = 1 << 0;
        /// Permite escrever no objeto (ex: send na porta).
        const WRITE   = 1 << 1;
        /// Permite executar/chamar o objeto (ex: syscall, func).
        const CALL    = 1 << 2;
        /// Permite conceder esta capability a outros (transferência).
        const GRANT   = 1 << 3;
        /// Permite deletar/revogar o objeto.
        const DESTROY = 1 << 4;

        /// Direitos totais (Root/Owner).
        const ALL     = Self::READ.bits() | Self::WRITE.bits() | Self::CALL.bits() | Self::GRANT.bits() | Self::DESTROY.bits();
    }
}

/// Handle para uma Capability no espaço do usuário.
/// É apenas um índice na tabela de capacidades do processo (CSpace).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CapHandle(pub u32);

impl CapHandle {
    pub const NULL: Self = Self(0);
}

/// A Capability real (Kernel Object).
/// Armazenada na CSpace do processo.
#[derive(Debug, Clone)]
pub struct Capability {
    /// O tipo do objeto apontado.
    pub object_type: CapType,
    /// Endereço ou ID do objeto no Kernel (ex: ponteiro para Port).
    pub object_addr: u64,
    /// Máscara de direitos.
    pub rights: CapRights,
}

impl Capability {
    pub fn new(object_type: CapType, object_addr: u64, rights: CapRights) -> Self {
        Self {
            object_type,
            object_addr,
            rights,
        }
    }

    pub fn null() -> Self {
        Self {
            object_type: CapType::Null,
            object_addr: 0,
            rights: CapRights::empty(),
        }
    }

    /// Verifica se a capability tem os direitos solicitados.
    #[inline]
    pub fn check(&self, required: CapRights) -> bool {
        self.rights.contains(required)
    }
}
