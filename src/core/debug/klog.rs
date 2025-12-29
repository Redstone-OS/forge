//! Sistema de Logging Simplificado
//!
//! Macros diretas para saída serial.
//! Sem traits complexas, apenas texto e u64.

/// Trait auxiliar para imprimir valores de tipos diferentes
pub trait SerialDebug {
    fn serial_debug(&self);
}

impl SerialDebug for u64 {
    fn serial_debug(&self) {
        crate::drivers::serial::write_str(" 0x");
        crate::drivers::serial::write_hex(*self);
    }
}

impl SerialDebug for usize {
    fn serial_debug(&self) {
        crate::drivers::serial::write_str(" 0x");
        crate::drivers::serial::write_hex(*self as u64);
    }
}

impl SerialDebug for u32 {
    fn serial_debug(&self) {
        crate::drivers::serial::write_str(" 0x");
        crate::drivers::serial::write_hex(*self as u64);
    }
}

impl SerialDebug for i32 {
    fn serial_debug(&self) {
        crate::drivers::serial::write_str(" 0x");
        crate::drivers::serial::write_hex(*self as u64);
    }
}

impl SerialDebug for &str {
    fn serial_debug(&self) {
        crate::drivers::serial::write_str(" ");
        crate::drivers::serial::write_str(self);
    }
}

/// Macro interna para escrita direta
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        // TODO: Implementar quando tivermos core::fmt
    };
}

/// Info Log
#[macro_export]
macro_rules! kinfo {
    ($msg:expr) => {
        $crate::drivers::serial::write_str("[INFO]  ");
        $crate::drivers::serial::write_str($msg);
        $crate::drivers::serial::write_str("\n");
    };
    ($msg:expr, $val:expr) => {
        $crate::drivers::serial::write_str("[INFO]  ");
        $crate::drivers::serial::write_str($msg);
        $crate::core::debug::klog::SerialDebug::serial_debug(&$val);
        $crate::drivers::serial::write_str("\n");
    };
}

/// Warn Log
#[macro_export]
macro_rules! kwarn {
    ($msg:expr) => {
        $crate::drivers::serial::write_str("[WARN]  ");
        $crate::drivers::serial::write_str($msg);
        $crate::drivers::serial::write_str("\n");
    };
    ($msg:expr, $val:expr) => {
        $crate::drivers::serial::write_str("[WARN]  ");
        $crate::drivers::serial::write_str($msg);
        $crate::core::debug::klog::SerialDebug::serial_debug(&$val);
        $crate::drivers::serial::write_str("\n");
    };
}

/// Error Log
#[macro_export]
macro_rules! kerror {
    ($msg:expr) => {
        $crate::drivers::serial::write_str("[ERROR] ");
        $crate::drivers::serial::write_str($msg);
        $crate::drivers::serial::write_str("\n");
    };
    ($msg:expr, $val:expr) => {
        $crate::drivers::serial::write_str("[ERROR] ");
        $crate::drivers::serial::write_str($msg);
        $crate::core::debug::klog::SerialDebug::serial_debug(&$val);
        $crate::drivers::serial::write_str("\n");
    };
}

/// Debug Log (Compilado apenas em debug/test ou se feature estiver ativa)
#[macro_export]
macro_rules! kdebug {
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        {
            $crate::drivers::serial::write_str("[DEBUG] ");
            $crate::drivers::serial::write_str($msg);
            $crate::drivers::serial::write_str("\n");
        }
    };
    ($msg:expr, $val:expr) => {
        #[cfg(debug_assertions)]
        {
            $crate::drivers::serial::write_str("[DEBUG] ");
            $crate::drivers::serial::write_str($msg);
            $crate::core::debug::klog::SerialDebug::serial_debug(&$val);
            $crate::drivers::serial::write_str("\n");
        }
    };
}
