/// Arquivo: core/debug/klog.rs
///
/// Propósito: Sistema centralizado de logging do kernel.
/// Substitui macros ad-hoc println/print. Envia saída para serial (e futuramente framebuffer/log buffer).
///
/// Detalhes de Implementação:
/// - Define níveis de severidade (Debug, Info, Warn, Error).
/// - Macros expandem para chamadas de função estáticas para reduzir inchaço de código.
/// - Depende de `crate::drivers::serial` para saída física.
// Sistema de logging do kernel

// Nota: Assumimos que crate::drivers::serial existe e tem write_str/write_hex.
// Se ainda não existir, isso causará erro de compilação até que o módulo drivers seja criado.
use crate::drivers::serial;

/// Nível de log
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

/// Emite uma linha de log
pub fn log(level: LogLevel, message: &str) {
    let prefix = match level {
        LogLevel::Debug => "[DEBUG] ",
        LogLevel::Info => "[INFO]  ",
        LogLevel::Warn => "[WARN]  ",
        LogLevel::Error => "[ERROR] ",
    };

    // SAFETY: Acesso à serial deve ser sincronizado internamente ou aceitamos corrupção em panic
    serial::write_str(prefix);
    serial::write_str(message);
    serial::write_str("\n");
}

/// Trait para valores que podem ser logados
pub trait LogValue {
    fn log(&self);
}

impl LogValue for u64 {
    fn log(&self) {
        serial::write_str(" 0x");
        serial::write_hex(*self);
    }
}

impl LogValue for i32 {
    fn log(&self) {
        serial::write_str(" 0x");
        serial::write_hex(*self as u64);
    }
}

impl LogValue for u32 {
    fn log(&self) {
        serial::write_str(" 0x");
        serial::write_hex(*self as u64);
    }
}

impl LogValue for isize {
    fn log(&self) {
        serial::write_str(" 0x");
        serial::write_hex(*self as u64);
    }
}

impl LogValue for usize {
    fn log(&self) {
        serial::write_str(" 0x");
        serial::write_hex(*self as u64);
    }
}

impl LogValue for &str {
    fn log(&self) {
        serial::write_str(" ");
        serial::write_str(self);
    }
}

/// Emite log com valor genérico
pub fn log_val(level: LogLevel, message: &str, value: impl LogValue) {
    let prefix = match level {
        LogLevel::Debug => "[DEBUG] ",
        LogLevel::Info => "[INFO]  ",
        LogLevel::Warn => "[WARN]  ",
        LogLevel::Error => "[ERROR] ",
    };

    serial::write_str(prefix);
    serial::write_str(message);
    value.log();
    serial::write_str("\n");
}

// Macros de conveniência

#[macro_export]
macro_rules! kinfo {
    ($msg:expr) => {
        $crate::core::debug::klog::log($crate::core::debug::klog::LogLevel::Info, $msg)
    };
    ($msg:expr, $val:expr) => {
        $crate::core::debug::klog::log_val($crate::core::debug::klog::LogLevel::Info, $msg, $val)
    };
}

#[macro_export]
macro_rules! kwarn {
    ($msg:expr) => {
        $crate::core::debug::klog::log($crate::core::debug::klog::LogLevel::Warn, $msg)
    };
    ($msg:expr, $val:expr) => {
        $crate::core::debug::klog::log_val($crate::core::debug::klog::LogLevel::Warn, $msg, $val)
    };
}

#[macro_export]
macro_rules! kerror {
    ($msg:expr) => {
        $crate::core::debug::klog::log($crate::core::debug::klog::LogLevel::Error, $msg)
    };
    ($msg:expr, $val:expr) => {
        $crate::core::debug::klog::log_val($crate::core::debug::klog::LogLevel::Error, $msg, $val)
    };
}

#[macro_export]
macro_rules! kdebug {
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        $crate::core::debug::klog::log($crate::core::debug::klog::LogLevel::Debug, $msg)
    };
    ($msg:expr, $val:expr) => {
        #[cfg(debug_assertions)]
        $crate::core::debug::klog::log_val($crate::core::debug::klog::LogLevel::Debug, $msg, $val)
    };
}
