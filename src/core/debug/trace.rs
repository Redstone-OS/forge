/// Arquivo: core/debug/trace.rs
///
/// Propósito: Tracing leve de execução.
/// Permite rastrear entrada e saída de escopos (funções) para debug.
///
/// Detalhes de Implementação:
/// - Usa padrão RAII (Resource Acquisition Is Initialization).
/// - Loga "ENTER" ao criar e "EXIT" ao destruir (Drop).
// Sistema de Tracing do Kernelcução
// Sistema de Tracing Simplificado
// Apenas macros diretas

#[macro_export]
macro_rules! ktrace {
    ($name:expr) => {
        #[cfg(debug_assertions)]
        {
            $crate::drivers::serial::write_str("[TRACE] ");
            $crate::drivers::serial::write_str($name);
            $crate::drivers::serial::write_str("\n");
        }
    };
    ($msg:expr, $val:expr) => {
        #[cfg(debug_assertions)]
        {
            $crate::drivers::serial::write_str("[TRACE] ");
            $crate::drivers::serial::write_str($msg);
            $crate::core::debug::klog::SerialDebug::serial_debug(&$val);
            $crate::drivers::serial::write_str("\n");
        }
    };
}
