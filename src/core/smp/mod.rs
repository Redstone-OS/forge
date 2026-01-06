//! # SMP - Symmetric Multi-Processing
//!
//! Gerenciamento de múltiplos núcleos de CPU.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                          SMP                                │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐    │
//! │  │  topology   │     │   percpu    │     │   bringup   │    │
//! │  │  CPU Info   │     │  Per-CPU    │     │  Wake APs   │    │
//! │  │  Detection  │     │  Variables  │     │  SIPI       │    │
//! │  └──────┬──────┘     └──────┬──────┘     └──────┬──────┘    │
//! │         │                   │                   │           │
//! │         └───────────────────┴───────────────────┘           │
//! │                             │                               │
//! │                             ▼                               │
//! │                    ┌─────────────┐                          │
//! │                    │     ipi     │                          │
//! │                    │  Inter-CPU  │                          │
//! │                    │  Messages   │                          │
//! │                    └─────────────┘                          │
//! │                                                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Fluxo de Inicialização
//!
//! ```text
//! ACPI Init
//!     │
//!     ▼
//! ┌─────────────────┐
//! │ Parse MADT      │  Descobre CPUs
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ topology::init  │  Preenche TOPOLOGY global
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ LAPIC Init      │  Habilita Local APIC do BSP
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ Per-CPU Init    │  Aloca stacks e estruturas
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ SMP Bringup     │  Acorda APs via SIPI
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ Scheduler       │  Todas as CPUs rodando
//! └─────────────────┘
//! ```

pub mod bringup;
pub mod ipi;
pub mod percpu;
pub mod topology;
pub mod trampoline;

pub use topology::{cpu_count, current_cpu, CpuInfo, TOPOLOGY};
