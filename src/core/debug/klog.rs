//! Sistema de logging do kernel

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
    
    serial::write(prefix);
    serial::write(message);
    serial::write("\n");
}

/// Emite log com valor hexadecimal
pub fn log_hex(level: LogLevel, message: &str, value: u64) {
    let prefix = match level {
        LogLevel::Debug => "[DEBUG] ",
        LogLevel::Info => "[INFO]  ",
        LogLevel::Warn => "[WARN]  ",
        LogLevel::Error => "[ERROR] ",
    };
    
    serial::write(prefix);
    serial::write(message);
    serial::write(" 0x");
    // serial::write_hex(value); // Removido pois driver serial nao tem write_hex exposto, usando placeholder
    // TODO: Implementar write_hex no serial ou usar formatação manual aqui se necessário
    serial::write("HEX_VAL_TODO"); 
    serial::write("\n");
}

// Macros de conveniência
#[macro_export]
macro_rules! kinfo {
    ($msg:expr) => {
        $crate::core::debug::klog::log(
            $crate::core::debug::klog::LogLevel::Info,
            $msg
        )
    };
    ($msg:expr, $val:expr) => {
        $crate::core::debug::klog::log_hex(
            $crate::core::debug::klog::LogLevel::Info,
            $msg,
            $val as u64
        )
    };
}

#[macro_export]
macro_rules! kwarn {
    ($msg:expr) => {
        $crate::core::debug::klog::log(
            $crate::core::debug::klog::LogLevel::Warn,
            $msg
        )
    };
    ($msg:expr, $val:expr) => {
        $crate::core::debug::klog::log_hex(
            $crate::core::debug::klog::LogLevel::Warn,
            $msg,
            $val as u64
        )
    };
}

#[macro_export]
macro_rules! kerror {
    ($msg:expr) => {
        $crate::core::debug::klog::log(
            $crate::core::debug::klog::LogLevel::Error,
            $msg
        )
    };
    ($msg:expr, $val:expr) => {
        $crate::core::debug::klog::log_hex(
            $crate::core::debug::klog::LogLevel::Error,
            $msg,
            $val as u64
        )
    };
}

#[macro_export]
macro_rules! kdebug {
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        $crate::core::debug::klog::log(
            $crate::core::debug::klog::LogLevel::Debug,
            $msg
        )
    };
    ($msg:expr, $val:expr) => {
        #[cfg(debug_assertions)]
        $crate::core::debug::klog::log_hex(
            $crate::core::debug::klog::LogLevel::Debug,
            $msg,
            $val as u64
        )
    };
}
