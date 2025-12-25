//! # Capability Primitives
//!
//! Este módulo define os "átomos" do modelo de segurança do Redstone: `Capability`, `rights` e `types`.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Kernel Object Reference:** Uma `Capability` aponta para um recurso real (ex: `Port`, `Frame`) e carrega metadados de acesso (`CapRights`).
//! - **Unforgeable Token:** O userspace opera apenas com `CapHandle` (inteiros). A `Capability` real vive em memória protegida do kernel.
//! - **Type Safety:** `CapType` garante que você não tente "chamar" uma página de memória ou "escrever" em uma interrupção.
//!
//! ## 🏗️ Arquitetura: Object-Capability Model
//! A estrutura `Capability` é a "chave mestra". Ela contém:
//! 1. **Tipo:** O que é? (`CapType`)
//! 2. **Endereço:** Onde está? (`object_addr`)
//! 3. **Direitos:** O que posso fazer? (`CapRights`)
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Bitflags for Rights:** O uso de `bitflags!` para `CapRights` permite composição eficiente (ex: `READ | WRITE`) e verificação O(1).
//! - **Strong Typing:** `CapHandle` é um tipo de tupla (`pub u32`) impedindo confusão com inteiros ou ponteiros nus.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Raw Pointers in Capability:** `object_addr` é um `u64`. Se o objeto apontado for desalocado (Use-After-Free), a Capability se torna um "Dangling Pointer".
//!   - *Correção Necessária:* O kernel precisa de um **Object Database** ou Reference Counting nas capabilities para garantir "Liveness".
//! - **Sem Badges:** Em seL4, capabilities podem ter um "Badge" (inteiro imutável) usado para identificar quem está chamando um servidor. Aqui, falta esse campo.
//!   - *Impacto:* Servidores não conseguem distinguir clientes facilmente sem criar um endpoint (Port) por cliente.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical)** Adicionar **Life-Cycle Management**.
//!   - *Problema:* Quando um objeto (ex: Thread) morre, quem limpa as capabilities que apontam pra ele?
//! - [ ] **TODO: (Feature)** Adicionar campo **Badge** à struct `Capability`.
//!   - *Caso de Uso:* Servidor de Filesystem usa o Badge para saber qual Client ID enviou a mensagem.
//! - [ ] **TODO: (Precision)** Refinar `CapType`.
//!   - *Ação:* Separar `Memory` em `Untyped` (memória crua) e `Frame` (memória mapeada), similar ao seL4.
//!
//! --------------------------------------------------------------------------------
//!
//!
//! Tipos de objetos que podem ser referenciados por uma Capability.
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
