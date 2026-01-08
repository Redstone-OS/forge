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
//!
//! ## Atomicidade
//!
//! Cada mensagem de log é escrita com um único lock, evitando interleaving
//! quando múltiplas CPUs fazem log simultaneamente.

use crate::drivers::comm::serial::SerialPort;

// =============================================================================
// SERIAL PRINT TRAIT (Para uso com lock)
// =============================================================================

/// Trait para tipos que podem ser impressos diretamente em um SerialPort.
///
/// Usado pelas macros de log para escrever valores com o lock mantido.
pub trait SerialPrintTo {
    fn serial_print_to(&self, serial: &mut SerialPort);
}

// Implementações para tipos numéricos

impl SerialPrintTo for u8 {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        s.write_hex(*self as u64);
    }
}

impl SerialPrintTo for u16 {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        s.write_hex(*self as u64);
    }
}

impl SerialPrintTo for u32 {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        s.write_hex(*self as u64);
    }
}

impl SerialPrintTo for u64 {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        s.write_hex(*self);
    }
}

impl SerialPrintTo for usize {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        s.write_hex(*self as u64);
    }
}

impl SerialPrintTo for i8 {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        if *self < 0 {
            s.write_byte(b'-');
            s.write_hex((-*self) as u64);
        } else {
            s.write_hex(*self as u64);
        }
    }
}

impl SerialPrintTo for i16 {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        if *self < 0 {
            s.write_byte(b'-');
            s.write_hex((-*self) as u64);
        } else {
            s.write_hex(*self as u64);
        }
    }
}

impl SerialPrintTo for i32 {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        if *self < 0 {
            s.write_byte(b'-');
            s.write_hex((-*self) as u64);
        } else {
            s.write_hex(*self as u64);
        }
    }
}

impl SerialPrintTo for i64 {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        if *self < 0 {
            s.write_byte(b'-');
            s.write_hex((-*self) as u64);
        } else {
            s.write_hex(*self as u64);
        }
    }
}

impl SerialPrintTo for isize {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        if *self < 0 {
            s.write_byte(b'-');
            s.write_hex((-*self) as u64);
        } else {
            s.write_hex(*self as u64);
        }
    }
}

impl SerialPrintTo for bool {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        if *self {
            s.write_str("true");
        } else {
            s.write_str("false");
        }
    }
}

impl SerialPrintTo for char {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        let mut buf = [0u8; 4];
        let encoded = self.encode_utf8(&mut buf);
        s.write_str(encoded);
    }
}

impl SerialPrintTo for &str {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        s.write_str(self);
    }
}

impl<T> SerialPrintTo for *const T {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        s.write_hex(*self as u64);
    }
}

impl<T> SerialPrintTo for *mut T {
    #[inline]
    fn serial_print_to(&self, s: &mut SerialPort) {
        s.write_hex(*self as u64);
    }
}

// =============================================================================
// MACROS DE LOG (ATÔMICAS)
// =============================================================================

/// Log de informação.
///
/// Cada mensagem é escrita atomicamente com um único lock.
#[macro_export]
macro_rules! kinfo {
    ($msg:expr) => {
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[INFO]  ");
            s.write_str($msg);
            s.write_str("\n");
        });
    };
    ($msg:expr, $val:expr) => {
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[INFO]  ");
            s.write_str($msg);
            s.write_str(" ");
            $crate::core::debug::klog::SerialPrintTo::serial_print_to(&$val, s);
            s.write_str("\n");
        });
    };
    ($msg:expr, $($val:expr),+ $(,)?) => {
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[INFO]  ");
            s.write_str($msg);
            $(
                s.write_str(" ");
                $crate::core::debug::klog::SerialPrintTo::serial_print_to(&$val, s);
            )+
            s.write_str("\n");
        });
    };
}

/// Log de aviso.
///
/// Cada mensagem é escrita atomicamente com um único lock.
#[macro_export]
macro_rules! kwarn {
    ($msg:expr) => {
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[WARN]  ");
            s.write_str($msg);
            s.write_str("\n");
        });
    };
    ($msg:expr, $val:expr) => {
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[WARN]  ");
            s.write_str($msg);
            s.write_str(" ");
            $crate::core::debug::klog::SerialPrintTo::serial_print_to(&$val, s);
            s.write_str("\n");
        });
    };
    ($msg:expr, $($val:expr),+ $(,)?) => {
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[WARN]  ");
            s.write_str($msg);
            $(
                s.write_str(" ");
                $crate::core::debug::klog::SerialPrintTo::serial_print_to(&$val, s);
            )+
            s.write_str("\n");
        });
    };
}

/// Log de erro.
///
/// Cada mensagem é escrita atomicamente com um único lock.
#[macro_export]
macro_rules! kerror {
    ($msg:expr) => {
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[ERROR] ");
            s.write_str($msg);
            s.write_str("\n");
        });
    };
    ($msg:expr, $val:expr) => {
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[ERROR] ");
            s.write_str($msg);
            s.write_str(" ");
            $crate::core::debug::klog::SerialPrintTo::serial_print_to(&$val, s);
            s.write_str("\n");
        });
    };
    ($msg:expr, $($val:expr),+ $(,)?) => {
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[ERROR] ");
            s.write_str($msg);
            $(
                s.write_str(" ");
                $crate::core::debug::klog::SerialPrintTo::serial_print_to(&$val, s);
            )+
            s.write_str("\n");
        });
    };
}

/// Log de debug (apenas em debug builds).
///
/// Cada mensagem é escrita atomicamente com um único lock.
#[macro_export]
macro_rules! kdebug {
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[DEBUG] ");
            s.write_str($msg);
            s.write_str("\n");
        });
    };
    ($msg:expr, $val:expr) => {
        #[cfg(debug_assertions)]
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[DEBUG] ");
            s.write_str($msg);
            s.write_str(" ");
            $crate::core::debug::klog::SerialPrintTo::serial_print_to(&$val, s);
            s.write_str("\n");
        });
    };
    ($msg:expr, $($val:expr),+ $(,)?) => {
        #[cfg(debug_assertions)]
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[DEBUG] ");
            s.write_str($msg);
            $(
                s.write_str(" ");
                $crate::core::debug::klog::SerialPrintTo::serial_print_to(&$val, s);
            )+
            s.write_str("\n");
        });
    };
}
