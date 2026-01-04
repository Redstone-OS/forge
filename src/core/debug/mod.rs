//! # Debug Subsystem
//!
//! Ferramentas de diagnóstico e logging do kernel.
//!
//! ## Componentes
//!
//! | Módulo    | Descrição                              |
//! |-----------|----------------------------------------|
//! | `klog`    | Macros de log (kinfo!, kerror!, etc)   |
//! | `kdebug`  | Breakpoints e assertions               |
//! | `oops`    | Erros recuperáveis                     |
//! | `stats`   | Contadores de performance              |
//! | `trace`   | Tracing de execução                    |
//! | `console` | Display visual (stub, desativado)      |
//!
//! ## Destinos de Log
//!
//! - **Serial**: Ativo (via QEMU ou hardware)
//! - **Display**: Desativado (stub)
//! - **Arquivo**: TODO futuro

pub mod console;
pub mod kdebug;
pub mod klog;
pub mod oops;
pub mod stats;
pub mod trace;

pub use stats::STATS;
