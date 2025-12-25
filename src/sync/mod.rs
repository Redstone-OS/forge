//! # Synchronization Primitives
//!
//! Este módulo fornece as abstrações necessárias para garantir a integridade de dados em um ambiente
//! de kernel concorrente (Multicore e Interrupt-driven).
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Mutual Exclusion:** Garante que apenas uma CPU (ou fluxo de execução) acesse um dado por vez.
//! - **Interior Mutability:** Permite modificar dados compartilhados (`static`) de forma segura (`Send` + `Sync`).
//!
//! ## 🏗️ Arquitetura: Spinlocks
//! Atualmente, o Redstone OS utiliza **Spinlocks** (`spin::Mutex`).
//! - **Comportamento:** Se o lock está ocupado, a thread entra em loop infinito (busy wait) até liberar.
//! - **Custo:** Alto uso de CPU durante a espera, mas zero overhead de escalonamento (não dorme).
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Lazy Initialization:** O uso de `spin::Lazy` resolve o problema do "Static Initialization Order Fiasco", permitindo
//!   inicializar globais complexos (como heaps e drivers) na primeira utilização.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Deadlock por Interrupção:** O `spin::Mutex` padrão **NÃO** desabilita interrupções.
//!   - *Cenário:* Thread A pega Lock X. Interrupção ocorre. Handler da Interrupção tenta pegar Lock X.
//!   - *Resultado:* Deadlock eterno na mesma CPU.
//! - **Priority Inversion:** Spinlocks simples não previnem inversão de prioridade (embora em SMP round-robin isso seja menos crítico hoje).
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Critical/Safety)** Implementar **IrqSafeMutex**.
//!   - *Meta:* Um wrapper que executa `cli` (disable interrupts) antes de pegar o lock e `sti` (restore) ao soltar.
//!   - *Necessário para:* Drivers, Scheduler e qualquer estrutura compartilhada com Interrupt Handlers.
//! - [ ] **TODO: (Debug)** Adicionar **Deadlock Detection**.
//!   - *Como:* O lock deve registrar qual CPU/Thread é dona dele. Se a mesma CPU tentar pegar 2x, panic imediato com backtrace.
//! - [ ] **TODO: (SMP)** Implementar **Ticket Locks** ou MCS Locks.
//!   - *Motivo:* Spinlocks simples não garantem justiça (fairness) em sistemas com muitos cores, podendo causar starvation de uma CPU.
//!
//! --------------------------------------------------------------------------------
//!
//! Re-exporta o Mutex da crate `spin` por enquanto.
//! Isso facilita mudar a implementação no futuro sem alterar o código consumidor.

// Re-exporta o Mutex da crate `spin` por enquanto.
// Isso facilita mudar a implementação no futuro sem alterar o código consumidor.
pub use spin::{Mutex, MutexGuard};

/// Wrapper para garantir inicialização preguiçosa segura.
pub use spin::Lazy;

pub mod test;
