//! # Core — Núcleo do Kernel
//!
//! Infraestrutura central agnóstica de hardware.
//!
//! ## Subsistemas
//!
//! | Módulo   | Responsabilidade                              |
//! |----------|-----------------------------------------------|
//! | `boot`   | Inicialização, kernel_main, panic handler     |
//! | `object` | Gerenciamento de objetos kernel (handles)     |
//! | `smp`    | Multiprocessamento (per-cpu, IPI, topology)   |
//! | `time`   | Relógios e timers                             |
//! | `work`   | Trabalho diferido (workqueues, tasklets)      |
//! | `power`  | Gerenciamento de energia (cpufreq, suspend)   |
//! | `debug`  | Logging, tracing, diagnóstico                 |

// =============================================================================
// BOOT — Inicialização do Sistema
// =============================================================================

pub mod boot;

// Re-export para acesso direto
pub use boot::entry::kernel_entry as kernel_main;
pub use boot::handoff::BootInfo;

// =============================================================================
// OBJECT — Sistema de Objetos do Kernel
// =============================================================================

pub mod object;

// =============================================================================
// SMP — Multiprocessamento Simétrico
// =============================================================================

pub mod smp;

// =============================================================================
// TIME — Tempo e Timers
// =============================================================================

pub mod time;

// =============================================================================
// WORK — Trabalho Diferido
// =============================================================================

pub mod work;

// =============================================================================
// POWER — Gerenciamento de Energia
// =============================================================================

pub mod power;

// =============================================================================
// DEBUG — Diagnóstico e Trace
// =============================================================================

pub mod debug;
