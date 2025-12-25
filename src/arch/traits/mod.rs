//! # Hardware Traits Interface (Contract)
//!
//! Este módulo define os **Contratos de Interface** (Traits) que qualquer arquitetura deve implementar
//! para ser suportada pelo Redstone OS.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Polimorfismo Estático:** Usa Traits para garantir que o *Kernel Core* chame métodos padronizados (`halt`, `current_id`), independentemente se o backend é `x86_64`, `arm64` ou `riscv`.
//! - **Segurança de Tipo:** Impede que o kernel chame funções inseguras ou inexistentes em uma plataforma específica.
//!
//! ## 🏗️ Estrutura
//! - `generic.rs` (hipotético futuro): Traits comuns (ex: `Arch`).
//! - `cpu.rs`: Operações básicas de processador (CPUID, Halt, Interrupts Control).
//!
//! ## 🔍 Análise Crítica
//!
//! ### ✅ Pontos Fortes
//! - **Simplicidade:** A trait `CpuOps` cobre o essencial para um kernel micro-modular (saber quem sou, parar, controlar interrupções).
//!
//! ### ⚠️ Pontos de Atenção
//! - **Acoplamento Temporal:** Algumas traits podem exigir inicialização prévia (ex: `current_id` pode precisar de APIC init). O contrato não explicita essas dependências.
//! - **Falta de `MmuOps`:** Atualmente a gerência de memória (PMM/VMM) está muito acoplada ao x86 (PML4 hardcoded). Deveria haver uma trait `PageTableOps`.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Architecture)** Criar `MmuOps` para abstrair tabelas de paginação.
//!   - *Motivo:* ARM64 usa tabelas diferentes (embora parecidas). RISC-V Sv39/Sv48 também. O VMM não pode depender de `cr3` diretamente.
//! - [ ] **TODO: (Cleanup)** Documentar requisitos de "Reentrância" e "Thread Safety" para cada método da trait.
//!   - *Motivo:* Métodos como `disable_interrupts` devem ser seguros para chamar de qualquer contexto (inclusive Exception Handlers).

pub mod cpu;

// Re-exportar para facilitar uso: `use crate::arch::traits::CpuOps;`
pub use cpu::CpuOps;
