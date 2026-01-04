//! # Debug Console (Stub)
//!
//! Console visual para debug - DESATIVADO.
//!
//! ## Status
//!
//! Este módulo está desativado. Todas as funções são no-op.
//!
//! ## Futuro
//!
//! Será ativável via parâmetro de boot:
//! - `debug_console=on` - Ativa console visual
//! - `debug_console=off` - Desativado (padrão)
//!
//! ## Por que desativado?
//!
//! - Lento (desenha pixel a pixel)
//! - Compete com display real
//! - Serial via QEMU é mais prático

/// Inicializa o console (no-op).
#[inline]
pub fn init() {
    // TODO: Implementar quando ativado via boot param
}

/// Limpa a tela (no-op).
#[inline]
pub fn clear() {}

/// Log de texto (no-op).
#[inline]
pub fn log(_msg: &str) {}

/// Log de texto com cor (no-op).
#[inline]
pub fn log_color(_msg: &str, _color: u32) {}

/// Log de texto + valor hex (no-op).
#[inline]
pub fn log_hex(_msg: &str, _value: u64) {}

/// Log de info (no-op).
#[inline]
pub fn info(_msg: &str) {}

/// Log de info com valor (no-op).
#[inline]
pub fn info_hex(_msg: &str, _value: u64) {}

/// Log de warning (no-op).
#[inline]
pub fn warn(_msg: &str) {}

/// Log de erro (no-op).
#[inline]
pub fn error(_msg: &str) {}

/// Verifica se está inicializado.
#[inline]
pub fn is_initialized() -> bool {
    false
}

/// Ativa exibição (no-op).
#[inline]
pub fn enable() {}

/// Desativa exibição (no-op).
#[inline]
pub fn disable() {}
