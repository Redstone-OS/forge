//! # Kernel Tracing
//!
//! Sistema de tracing para debug de execução.

/// Macro de trace (apenas em debug builds).
#[macro_export]
macro_rules! ktrace {
    ($name:expr) => {
        #[cfg(debug_assertions)]
        {
            $crate::drivers::comm::serial::write_str("[TRACE] ");
            $crate::drivers::comm::serial::write_str($name);
            $crate::drivers::comm::serial::write_str("\n");
        }
    };
    ($msg:expr, $val:expr) => {
        #[cfg(debug_assertions)]
        {
            $crate::drivers::comm::serial::write_str("[TRACE] ");
            $crate::drivers::comm::serial::write_str($msg);
            $crate::drivers::comm::serial::write_str(" ");
            $crate::core::debug::klog::SerialPrint::serial_print(&$val);
            $crate::drivers::comm::serial::write_str("\n");
        }
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
        crate::drivers::comm::serial::write_str("[TRACE] ENTER ");
        crate::drivers::comm::serial::write_str(name);
        crate::drivers::comm::serial::write_str("\n");
        Self { name }
    }
}

#[cfg(debug_assertions)]
impl Drop for TraceGuard {
    fn drop(&mut self) {
        crate::drivers::comm::serial::write_str("[TRACE] EXIT ");
        crate::drivers::comm::serial::write_str(self.name);
        crate::drivers::comm::serial::write_str("\n");
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
