//! # Security Subsystem (Capabilities)
//!
//! O Redstone OS rejeita o modelo de segurança baseado em identidades globais (ACLs, UID 0/Root)
//! em favor de um modelo baseado em **Capabilities** (Tokens de Permissão).
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Fine-Grained Access Control:** Segurança definida por "o que você tem" (token), não "quem você é".
//! - **Kernel Object Protection:** Todo acesso a recursos (portas, memória, IRQs) requer um handle válido.
//! - **Decentralization:** A segurança é distribuída. O kernel apenas valida tokens; a política é definida por quem detém o token.
//!
//! ## 🏗️ Arquitetura: Capability-Based Security
//! - **C-Space (Capability Space):** Cada processo tem sua própria tabela de capabilities, isolada das demais.
//! - **CapHandle:** Um inteiro (index) usado pelo userspace para referenciar uma capability em seu C-Space.
//! - **Delegation:** Capabilities podem ser transferidas entre processos via IPC, permitindo padrões seguros como *Least Privilege*.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Zero Trust:** Nanhuma syscall "mágica" funciona sem um handle explícito.
//! - **Imutabilidade:** As definições de `CapRights` são estáticas ("bitflags"), facilitando auditoria.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Falta de CNode/CSpace:** A tabela de capabilities real (`CSpace`) ainda não está implementada.
//!   Atualmente o `Task` tem um placeholder `HandleTable`, mas falta a lógica hierárquica (CNodes) do seL4.
//! - **Revogação Inexistente:** Não há mecanismo para revogar uma capability que foi delegada (Grant). Isso quebra o princípio de controle total.
//! - **Derivação:** Não é possível criar uma capability "mais fraca" a partir de uma forte (ex: criar Read-Only a partir de Read-Write).
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical)** Implementar **CSpace / CNodes**.
//!   - *Meta:* Estrutura de dados eficiente (Radix Tree ou Multi-level Table) para armazenar capabilities por processo.
//! - [ ] **TODO: (Feature)** Implementar **Revocation (Badge/Epoch)**.
//!   - *Motivo:* Permitir que um servidor cancele o acesso de um cliente a qualquer momento.
//! - [ ] **TODO: (Security)** Implementar **Derived Capabilities (Minting)**.
//!   - *Cenário:* Processo A tem RW em uma porta, e quer passar apenas RO para processo B.
//! - [ ] **TODO: (Arch)** Definir **Object Capability Model** para Hardware (MMIO).
//!   - *Meta:* Drivers só acessam regiões de memória específicas via capabilities, sem acesso direto ao mapa físico.
//!

pub mod capability;
pub mod test;

pub use capability::{CapHandle, CapRights, CapType, Capability};

// TODO: Implementar CNode / CSpace (Tabela de Capabilities)
