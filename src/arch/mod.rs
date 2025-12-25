//! # Hardware Abstraction Layer (HAL)
//!
//! O módulo `arch` atua como a **única** ponte entre o *Kernel Core* (lógica agnóstica) e o hardware real.
//! Toda interação com registradores, instruções privilegiadas e controle de CPU deve passar por aqui.
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Isolamento:** O resto do kernel (`forge::core`, `forge::mm`, `forge::sched`) **não deve** saber em qual CPU está rodando.
//! - **Abstração:** Define traits (em `traits/`) que as implementações (ex: `x86_64/`) devem satisfazer.
//! - **Seleção de Plataforma:** Usa `cfg` attributes para compilar apenas o código da arquitetura alvo.
//!
//! ## 🏗️ Arquitetura e Fluxo
//! 1. O `Kernel Core` importa `crate::arch::Cpu`.
//! 2. `Cpu` é um *type alias* para a implementação concreta (ex: `x86_64::cpu::X64Cpu`).
//! 3. Funções como `Cpu::halt()` ou `Cpu::disable_interrupts()` são traduzidas para instruções assembly específicas (ex: `hlt`, `cli`).
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Separação Clara:** A estrutura de diretórios (`traits/` vs `x86_64/`) segue boas práticas de Portabilidade.
//! - **Type Safety:** O uso de Traits reduz o risco de chamar código específico de plataforma onde não deve.
//!
//! ### ⚠️ Pontos de Atenção
//! - **Dependência de Macros:** Algumas partes do kernel ainda podem estar usando macros que assumem x86 (verificar logs/prints).
//! - **Vazamento de Abstração:** Se o `bootinfo` passar estruturas específicas de hardware (como mapa de memória x86-only), a abstração falha na inicialização.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Roadmap)** Adicionar suporte inicial a `aarch64` (ARM64) para validar a abstração.
//!   - *Motivo:* Garantir que a HAL não está "viciada" em conceitos x86 (como Port IO vs MMIO).
//! - [ ] **TODO: (Performance)** Avaliar overhead de monomorfização das traits.
//!   - *Impacto:* Em kernels monolíticos, chamadas indiretas (dyn) são custosas; aqui usamos dispatch estático (impl trait), o que é bom, mas precisa ser vigiado.
//! - [ ] **TODO: (Refactor)** Mover definições de `PAGE_SIZE` para cá.
//!   - *Motivo:* 4KiB é padrão x86, mas outras archs usam 16KiB ou 64KiB. O MMCore não deve assumir 4096 hardcoded.

pub mod traits;

// Seleção de Arquitetura: x86_64
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64 as platform;

// Re-exports globais para o kernel usar
// Exemplo: arch::cpu::halt();
pub use platform::Cpu;
pub use traits::*;

pub mod test;
