/// Arquivo: core/debug/mod.rs
///
/// Propósito: Módulo de diagnóstico e depuração.
/// Fornece ferramentas para inspeção, logging, tracing e estatísticas do kernel.
///
/// Módulos contidos:
/// - `klog`: Macros de logging (kinfo, kerror, etc).
/// - `kdebug`: Utilitários de debug (breakpoints, assertions).
/// - `oops`: Tratamento de erros recuperáveis.
/// - `stats`: Contadores globais de performance/eventos.
/// - `trace`: Sistema de tracing leve.

pub mod klog;
pub mod kdebug;
pub mod oops;
pub mod stats;
pub mod trace;
