//! # Kernel Core Logic
//!
//! O módulo `core` é o **coração agnóstico** do kernel. Ele contém a lógica de infraestrutura
//! que não depende diretamente de hardware (diferente de `arch`) e nem implementa políticas de alto nível (diferente de `sched` ou `ipc`).
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Orquestração de Boot:** `entry.rs` define a sequência exata de inicialização (MM -> Drivers -> Sched).
//! - **Infraestrutura Crítica:** Logs (`logging`) e Handles (`handle`).
//! - **Contrato com Bootloader:** `handoff.rs` define a ABI binária (Structs `#[repr(C)]`) com o Ignite.
//! - **Panic Handling:** O *último recurso*. O kernel **NÃO DEVE** panicar em operação normal.
//!
//! ## 📜 Política de Integridade (Zero Panic Policy)
//! O Redstone OS segue uma política rígida onde **Panics são inaceitáveis** em runtime.
//! - `unwrap()`, `expect()` e `panic!()` são proibidos fora da fase de inicialização (`init`).
//! - Erros devem ser propagados via `Result`.
//! - Se um serviço falha, ele deve ser reiniciado, não derrubar o kernel.
//!
//! ## 🏗️ Sub-Módulos
//!
//! ## 🏗️ Sub-Módulos
//!
//! | Módulo    | Responsabilidade |
//! |-----------|------------------|
//! | `entry`   | Ponto de entrada Rust. Gerencia o ciclo de vida do boot até o `spawn_init`. |
//! | `handoff` | Definições de estruturas compartilhadas com o Bootloader (BootInfo, MemoryMap). |
//! | `logging` | Sistema de logs estruturado, thread-safe e IRQ-safe. |
//! | `panic`   | Handler de "tela azul" (ou serial output) para erros irrecuperáveis. |
//! | `elf`     | Loader básico de executáveis (usado para carregar o `/init` inicial). |
//! | `handle`  | Gerenciamento de recursos (descritores) para processos. |
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Centralização do Boot:** Ter um único arquivo (`entry.rs`) controlando a ordem de init facilita muito o debug de boot.
//! - **Logging Robusto:** O logger lida bem com concorrência e interrupções, essencial para debugar falhas de SMP.
//!
//! ### ⚠️ Pontos de Atenção
//! - **Loader ELF no Core:** O parser ELF (`elf.rs`) em `core` é questionável. Carregamento de binários geralmente pertence a um subsistema de execução (`sys` ou `loader`).
//!   - *Risco:* Aumenta a superfície de ataque do core se o parser tiver bugs.
//! - **Handle Table Simplista:** A implementação atual de Handles pode não escalar bem para milhares de recursos por processo.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Refactor)** Mover `elf.rs` para um crate ou módulo separado `kernel::loader`.
//!   - *Motivo:* "Core" deve ser apenas o essencial para o kernel existir. Carregar ELF é uma feature.
//! - [ ] **TODO: (Security)** Auditar o parser ELF contra buffer overflows e loops infinitos.
//!   - *Impacto:* Um `/init` malicioso ou corrompido não deve conseguir crashar o kernel via parser.
//! - [ ] **TODO: (Architecture)** Abstrair o mecanismo de Shutdown/Reboot.
//!   - *Motivo:* Atualmente o `panic` apenas trava (hang). Precisamos de reset via ACPI ou controlador de teclado.

pub mod elf;
pub mod entry;
pub mod handle;
pub mod handoff;
pub mod logging;
pub mod panic;
pub mod test;
