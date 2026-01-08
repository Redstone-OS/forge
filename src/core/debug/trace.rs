//! # Kernel Tracing
//!
//! Sistema de tracing para debug de execução.
//!
//! ## Atomicidade
//!
//! Cada trace é escrito atomicamente com um único lock.

/// Macro de trace (apenas em debug builds).
///
/// Cada mensagem é escrita atomicamente com um único lock.
#[macro_export]
macro_rules! ktrace {
    ($name:expr) => {
        #[cfg(debug_assertions)]
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[TRACE] ");
            s.write_str($name);
            s.write_str("\n");
        });
    };
    ($msg:expr, $val:expr) => {
        #[cfg(debug_assertions)]
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[TRACE] ");
            s.write_str($msg);
            s.write_str(" ");
            $crate::core::debug::klog::SerialPrintTo::serial_print_to(&$val, s);
            s.write_str("\n");
        });
    };
    ($msg:expr, $($val:expr),+ $(,)?) => {
        #[cfg(debug_assertions)]
        $crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[TRACE] ");
            s.write_str($msg);
            $(
                s.write_str(" ");
                $crate::core::debug::klog::SerialPrintTo::serial_print_to(&$val, s);
            )+
            s.write_str("\n");
        });
    };
}

/// Guard RAII para tracing de escopo.
///
/// Loga "ENTER" ao criar e "EXIT" ao dropar.
#[cfg(debug_assertions)]
pub struct TraceGuard {
    name: &'static str,
}

#[cfg(debug_assertions)]
impl TraceGuard {
    /// Cria novo trace guard.
    #[inline]
    pub fn new(name: &'static str) -> Self {
        crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[TRACE] ENTER ");
            s.write_str(name);
            s.write_str("\n");
        });
        Self { name }
    }
}

#[cfg(debug_assertions)]
impl Drop for TraceGuard {
    fn drop(&mut self) {
        crate::drivers::comm::serial::with_lock(|s| {
            s.write_str("[TRACE] EXIT ");
            s.write_str(self.name);
            s.write_str("\n");
        });
    }
}

/// Macro para criar trace guard de escopo.
#[macro_export]
macro_rules! ktrace_scope {
    ($name:expr) => {
        #[cfg(debug_assertions)]
        let _guard = $crate::core::debug::trace::TraceGuard::new($name);
    };
}
