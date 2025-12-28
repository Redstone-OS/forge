/// Arquivo: core/debug/trace.rs
///
/// Propósito: Tracing leve de execução.
/// Permite rastrear entrada e saída de escopos (funções) para debug.
///
/// Detalhes de Implementação:
/// - Usa padrão RAII (Resource Acquisition Is Initialization).
/// - Loga "ENTER" ao criar e "EXIT" ao destruir (Drop).

//! Tracing de execução

use crate::core::debug::klog::{log, LogLevel};

pub struct ScopedTrace {
    name: &'static str,
}

impl ScopedTrace {
    pub fn new(name: &'static str) -> Self {
        log(LogLevel::Debug, "ENTER:");
        // TODO: Melhorar formatação quando tivermos alloc::format! ou similar
        log(LogLevel::Debug, name);
        Self { name }
    }
}

impl Drop for ScopedTrace {
    fn drop(&mut self) {
        log(LogLevel::Debug, "EXIT: ");
        log(LogLevel::Debug, self.name);
    }
}

/// Cria um rastreador de escopo.
/// O valor retornado deve ser atribuído a uma variável local (_guard) para viver até o fim do escopo.
pub fn trace_scope(name: &'static str) -> ScopedTrace {
    ScopedTrace::new(name)
}

#[macro_export]
macro_rules! ktrace {
    ($name:expr) => {
        let _trace_guard = $crate::core::debug::trace::trace_scope($name);
    };
    ($name:expr, $($arg:tt)*) => {
        let _trace_guard = $crate::core::debug::trace::trace_scope($name);
    };
}
