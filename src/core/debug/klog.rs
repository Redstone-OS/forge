//! # Kernel Logging
//!
//! Sistema de logging via serial para debug.
//!
//! ## Macros Disponíveis
//!
//! ```rust
//! kinfo!("Mensagem");           // [INFO] Mensagem
//! kinfo!("Valor:", 42);         // [INFO] Valor: 0x2A
//! kwarn!("Atenção");            // [WARN] Atenção
//! kerror!("Erro");              // [ERROR] Erro
//! kdebug!("Debug");             // [DEBUG] Debug (só em debug builds)
//! ```
//!
//! ## Performance
//!
//! Logs são simples e diretos - sem formatação complexa para manter
//! o overhead mínimo. Valores numéricos são impressos em hexadecimal.

// =============================================================================
// SERIAL PRINT TRAIT
// =============================================================================

/// Trait para tipos que podem ser impressos via serial.
pub trait SerialPrint {
    fn serial_print(&self);
}

// Implementações para tipos numéricos

impl SerialPrint for u8 {
    #[inline]
    fn serial_print(&self) {
        crate::drivers::comm::serial::write_hex(*self as u64);
    }
}

impl SerialPrint for u16 {
    #[inline]
    fn serial_print(&self) {
        crate::drivers::comm::serial::write_hex(*self as u64);
    }
}

impl SerialPrint for u32 {
    #[inline]
    fn serial_print(&self) {
        crate::drivers::comm::serial::write_hex(*self as u64);
    }
}

impl SerialPrint for u64 {
    #[inline]
    fn serial_print(&self) {
        crate::drivers::comm::serial::write_hex(*self);
    }
}

impl SerialPrint for usize {
    #[inline]
    fn serial_print(&self) {
        crate::drivers::comm::serial::write_hex(*self as u64);
    }
}

impl SerialPrint for i8 {
    #[inline]
    fn serial_print(&self) {
        if *self < 0 {
            crate::drivers::comm::serial::write_byte(b'-');
            crate::drivers::comm::serial::write_hex((-*self) as u64);
        } else {
            crate::drivers::comm::serial::write_hex(*self as u64);
        }
    }
}

impl SerialPrint for i16 {
    #[inline]
    fn serial_print(&self) {
        if *self < 0 {
            crate::drivers::comm::serial::write_byte(b'-');
            crate::drivers::comm::serial::write_hex((-*self) as u64);
        } else {
            crate::drivers::comm::serial::write_hex(*self as u64);
        }
    }
}

impl SerialPrint for i32 {
    #[inline]
    fn serial_print(&self) {
        if *self < 0 {
            crate::drivers::comm::serial::write_byte(b'-');
            crate::drivers::comm::serial::write_hex((-*self) as u64);
        } else {
            crate::drivers::comm::serial::write_hex(*self as u64);
        }
    }
}

impl SerialPrint for i64 {
    #[inline]
    fn serial_print(&self) {
        if *self < 0 {
            crate::drivers::comm::serial::write_byte(b'-');
            crate::drivers::comm::serial::write_hex((-*self) as u64);
        } else {
            crate::drivers::comm::serial::write_hex(*self as u64);
        }
    }
}

impl SerialPrint for isize {
    #[inline]
    fn serial_print(&self) {
        if *self < 0 {
            crate::drivers::comm::serial::write_byte(b'-');
            crate::drivers::comm::serial::write_hex((-*self) as u64);
        } else {
            crate::drivers::comm::serial::write_hex(*self as u64);
        }
    }
}

impl SerialPrint for bool {
    #[inline]
    fn serial_print(&self) {
        if *self {
            crate::drivers::comm::serial::write_str("true");
        } else {
            crate::drivers::comm::serial::write_str("false");
        }
    }
}

impl SerialPrint for char {
    #[inline]
    fn serial_print(&self) {
        let mut buf = [0u8; 4];
        let s = self.encode_utf8(&mut buf);
        crate::drivers::comm::serial::write_str(s);
    }
}

impl SerialPrint for &str {
    #[inline]
    fn serial_print(&self) {
        crate::drivers::comm::serial::write_str(self);
    }
}

impl<T> SerialPrint for *const T {
    #[inline]
    fn serial_print(&self) {
        crate::drivers::comm::serial::write_hex(*self as u64);
    }
}

impl<T> SerialPrint for *mut T {
    #[inline]
    fn serial_print(&self) {
        crate::drivers::comm::serial::write_hex(*self as u64);
    }
}

// =============================================================================
// MACROS DE LOG
// =============================================================================

/// Log de informação.
#[macro_export]
macro_rules! kinfo {
    ($msg:expr) => {
        $crate::drivers::comm::serial::write_str("[INFO]  ");
        $crate::drivers::comm::serial::write_str($msg);
        $crate::drivers::comm::serial::write_str("\n");
    };
    ($msg:expr, $val:expr) => {
        $crate::drivers::comm::serial::write_str("[INFO]  ");
        $crate::drivers::comm::serial::write_str($msg);
        $crate::drivers::comm::serial::write_str(" ");
        $crate::core::debug::klog::SerialPrint::serial_print(&$val);
        $crate::drivers::comm::serial::write_str("\n");
    };
    ($msg:expr, $($val:expr),+ $(,)?) => {
        $crate::drivers::comm::serial::write_str("[INFO]  ");
        $crate::drivers::comm::serial::write_str($msg);
        $(
            $crate::drivers::comm::serial::write_str(" ");
            $crate::core::debug::klog::SerialPrint::serial_print(&$val);
        )+
        $crate::drivers::comm::serial::write_str("\n");
    };
}

/// Log de aviso.
#[macro_export]
macro_rules! kwarn {
    ($msg:expr) => {
        $crate::drivers::comm::serial::write_str("[WARN]  ");
        $crate::drivers::comm::serial::write_str($msg);
        $crate::drivers::comm::serial::write_str("\n");
    };
    ($msg:expr, $val:expr) => {
        $crate::drivers::comm::serial::write_str("[WARN]  ");
        $crate::drivers::comm::serial::write_str($msg);
        $crate::drivers::comm::serial::write_str(" ");
        $crate::core::debug::klog::SerialPrint::serial_print(&$val);
        $crate::drivers::comm::serial::write_str("\n");
    };
    ($msg:expr, $($val:expr),+ $(,)?) => {
        $crate::drivers::comm::serial::write_str("[WARN]  ");
        $crate::drivers::comm::serial::write_str($msg);
        $(
            $crate::drivers::comm::serial::write_str(" ");
            $crate::core::debug::klog::SerialPrint::serial_print(&$val);
        )+
        $crate::drivers::comm::serial::write_str("\n");
    };
}

/// Log de erro.
#[macro_export]
macro_rules! kerror {
    ($msg:expr) => {
        $crate::drivers::comm::serial::write_str("[ERROR] ");
        $crate::drivers::comm::serial::write_str($msg);
        $crate::drivers::comm::serial::write_str("\n");
    };
    ($msg:expr, $val:expr) => {
        $crate::drivers::comm::serial::write_str("[ERROR] ");
        $crate::drivers::comm::serial::write_str($msg);
        $crate::drivers::comm::serial::write_str(" ");
        $crate::core::debug::klog::SerialPrint::serial_print(&$val);
        $crate::drivers::comm::serial::write_str("\n");
    };
    ($msg:expr, $($val:expr),+ $(,)?) => {
        $crate::drivers::comm::serial::write_str("[ERROR] ");
        $crate::drivers::comm::serial::write_str($msg);
        $(
            $crate::drivers::comm::serial::write_str(" ");
            $crate::core::debug::klog::SerialPrint::serial_print(&$val);
        )+
        $crate::drivers::comm::serial::write_str("\n");
    };
}

/// Log de debug (apenas em debug builds).
#[macro_export]
macro_rules! kdebug {
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        {
            $crate::drivers::comm::serial::write_str("[DEBUG] ");
            $crate::drivers::comm::serial::write_str($msg);
            $crate::drivers::comm::serial::write_str("\n");
        }
    };
    ($msg:expr, $val:expr) => {
        #[cfg(debug_assertions)]
        {
            $crate::drivers::comm::serial::write_str("[DEBUG] ");
            $crate::drivers::comm::serial::write_str($msg);
            $crate::drivers::comm::serial::write_str(" ");
            $crate::core::debug::klog::SerialPrint::serial_print(&$val);
            $crate::drivers::comm::serial::write_str("\n");
        }
    };
    ($msg:expr, $($val:expr),+ $(,)?) => {
        #[cfg(debug_assertions)]
        {
            $crate::drivers::comm::serial::write_str("[DEBUG] ");
            $crate::drivers::comm::serial::write_str($msg);
            $(
                $crate::drivers::comm::serial::write_str(" ");
                $crate::core::debug::klog::SerialPrint::serial_print(&$val);
            )+
            $crate::drivers::comm::serial::write_str("\n");
        }
    };
}
