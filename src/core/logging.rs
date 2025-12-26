// =============================================================================
// KERNEL LOGGING SYSTEM - ZERO OVERHEAD
// =============================================================================
//
// Sistema de logging do Redstone OS Kernel com custo ZERO em release.
//
// ARQUITETURA:
// Este sistema foi projetado para ser completamente removível em release:
// - Usa features do Cargo para compile-time filtering
// - Com feature "no_logs", TODOS os macros viram expressões vazias
// - SEM core::fmt - Evita geração de código SSE/AVX
// - SEM alocação - Apenas strings literais
// - Escreve APENAS na serial (não no console de vídeo)
//
// NÍVEIS DE LOG (do mais crítico ao menos):
// - ERROR: Erros fatais ou críticos
// - WARN:  Situações suspeitas mas recuperáveis
// - INFO:  Fluxo normal de execução
// - DEBUG: Informações de debugging
// - TRACE: Detalhes extremos (cada operação)
//
// FEATURES:
// - no_logs:   Remove 100% dos logs (custo zero no binário)
// - log_info:  Apenas ERROR, WARN, INFO
// - log_trace: Todos os níveis (padrão)
//
// COMO USAR (NOVA SINTAXE):
//
// ANTES (antigo - NÃO usar):
//   kinfo!("Valor: {:#x}", some_value);
//
// DEPOIS (novo - usar):
//   kinfo!("(PMM) Inicializando...");          // Apenas string
//   kinfo!("(PMM) Addr=", 0x1000);             // String + hex
//   klog!("Valor=", addr, " Size=", size);     // Múltiplos valores
//
// =============================================================================

use crate::drivers::serial;

// =============================================================================
// PREFIXOS COM CORES ANSI
// =============================================================================
//
// Cores ANSI para terminais que suportam (como o QEMU serial console).
// Cada prefixo inclui: código de cor + texto + reset de cor.
//
// Formato: \x1b[<código>m  onde:
//   1;31 = Bold Red
//   1;33 = Bold Yellow
//   32   = Green
//   36   = Cyan
//   35   = Magenta
//   0    = Reset
//

pub const P_ERROR: &str = "\x1b[1;31m[ERRO]\x1b[0m ";
pub const P_WARN: &str = "\x1b[1;33m[WARN]\x1b[0m ";
pub const P_INFO: &str = "\x1b[32m[INFO]\x1b[0m ";
pub const P_DEBUG: &str = "\x1b[36m[DEBG]\x1b[0m ";
pub const P_TRACE: &str = "\x1b[35m[TRAC]\x1b[0m ";

// =============================================================================
// MACROS DE LOG - NÍVEL ERROR
// =============================================================================
//
// kerror! - Sempre ativo (exceto com no_logs)
// Usado para erros críticos que podem causar crash.
//

#[cfg(not(feature = "no_logs"))]
#[macro_export]
macro_rules! kerror {
    // Apenas string literal
    ($msg:expr) => {{
        $crate::drivers::serial::emit_str($crate::core::logging::P_ERROR);
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_nl();
    }};
    // String + valor hex
    ($msg:expr, $val:expr) => {{
        $crate::drivers::serial::emit_str($crate::core::logging::P_ERROR);
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_hex($val as u64);
        $crate::drivers::serial::emit_nl();
    }};
}

#[cfg(feature = "no_logs")]
#[macro_export]
macro_rules! kerror {
    ($($t:tt)*) => {{}};
}

// =============================================================================
// MACROS DE LOG - NÍVEL WARN
// =============================================================================
//
// kwarn! - Ativo exceto com no_logs
// Usado para situações suspeitas mas recuperáveis.
//

#[cfg(not(feature = "no_logs"))]
#[macro_export]
macro_rules! kwarn {
    ($msg:expr) => {{
        $crate::drivers::serial::emit_str($crate::core::logging::P_WARN);
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_nl();
    }};
    ($msg:expr, $val:expr) => {{
        $crate::drivers::serial::emit_str($crate::core::logging::P_WARN);
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_hex($val as u64);
        $crate::drivers::serial::emit_nl();
    }};
}

#[cfg(feature = "no_logs")]
#[macro_export]
macro_rules! kwarn {
    ($($t:tt)*) => {{}};
}

// =============================================================================
// MACROS DE LOG - NÍVEL INFO
// =============================================================================
//
// kinfo! - Ativo exceto com no_logs
// Usado para eventos importantes do fluxo normal.
//

#[cfg(not(feature = "no_logs"))]
#[macro_export]
macro_rules! kinfo {
    ($msg:expr) => {{
        $crate::drivers::serial::emit_str($crate::core::logging::P_INFO);
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_nl();
    }};
    ($msg:expr, $val:expr) => {{
        $crate::drivers::serial::emit_str($crate::core::logging::P_INFO);
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_hex($val as u64);
        $crate::drivers::serial::emit_nl();
    }};
}

#[cfg(feature = "no_logs")]
#[macro_export]
macro_rules! kinfo {
    ($($t:tt)*) => {{}};
}

// =============================================================================
// MACROS DE LOG - NÍVEL DEBUG
// =============================================================================
//
// kdebug! - Ativo apenas com log_trace ou log_info
// Usado para informações de debugging.
//

#[cfg(any(feature = "log_trace", feature = "log_info"))]
#[macro_export]
macro_rules! kdebug {
    ($msg:expr) => {{
        $crate::drivers::serial::emit_str($crate::core::logging::P_DEBUG);
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_nl();
    }};
    ($msg:expr, $val:expr) => {{
        $crate::drivers::serial::emit_str($crate::core::logging::P_DEBUG);
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_hex($val as u64);
        $crate::drivers::serial::emit_nl();
    }};
}

#[cfg(not(any(feature = "log_trace", feature = "log_info")))]
#[macro_export]
macro_rules! kdebug {
    ($($t:tt)*) => {{}};
}

// =============================================================================
// MACROS DE LOG - NÍVEL TRACE
// =============================================================================
//
// ktrace! - Ativo apenas com log_trace
// Usado para detalhes extremos de cada operação.
//

#[cfg(feature = "log_trace")]
#[macro_export]
macro_rules! ktrace {
    ($msg:expr) => {{
        $crate::drivers::serial::emit_str($crate::core::logging::P_TRACE);
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_nl();
    }};
    ($msg:expr, $val:expr) => {{
        $crate::drivers::serial::emit_str($crate::core::logging::P_TRACE);
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_hex($val as u64);
        $crate::drivers::serial::emit_nl();
    }};
}

#[cfg(not(feature = "log_trace"))]
#[macro_export]
macro_rules! ktrace {
    ($($t:tt)*) => {{}};
}

// =============================================================================
// MACROS AUXILIARES
// =============================================================================

/// klog! - Log genérico sem prefixo de nível.
///
/// Útil para construir logs complexos com múltiplos valores.
///
/// # Uso
/// ```rust
/// klog!("Addr=", addr);                    // String + hex
/// klog!("Start=", start, " End=", end);    // Múltiplos
/// ```
#[cfg(not(feature = "no_logs"))]
#[macro_export]
macro_rules! klog {
    // Apenas string
    ($msg:expr) => {{
        $crate::drivers::serial::emit_str($msg);
    }};
    // String + hex
    ($msg:expr, $val:expr) => {{
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_hex($val as u64);
    }};
    // String + hex + string
    ($msg1:expr, $val:expr, $msg2:expr) => {{
        $crate::drivers::serial::emit_str($msg1);
        $crate::drivers::serial::emit_hex($val as u64);
        $crate::drivers::serial::emit_str($msg2);
    }};
    // String + hex + string + hex
    ($msg1:expr, $val1:expr, $msg2:expr, $val2:expr) => {{
        $crate::drivers::serial::emit_str($msg1);
        $crate::drivers::serial::emit_hex($val1 as u64);
        $crate::drivers::serial::emit_str($msg2);
        $crate::drivers::serial::emit_hex($val2 as u64);
    }};
}

#[cfg(feature = "no_logs")]
#[macro_export]
macro_rules! klog {
    ($($t:tt)*) => {{}};
}

/// knl! - Emite apenas newline.
#[cfg(not(feature = "no_logs"))]
#[macro_export]
macro_rules! knl {
    () => {{
        $crate::drivers::serial::emit_nl();
    }};
}

#[cfg(feature = "no_logs")]
#[macro_export]
macro_rules! knl {
    () => {{}};
}

// =============================================================================
// MACROS DE STATUS (OK/FAIL)
// =============================================================================

/// kok! - Log de sucesso (prefixo verde [OK]).
#[cfg(not(feature = "no_logs"))]
#[macro_export]
macro_rules! kok {
    ($msg:expr) => {{
        $crate::drivers::serial::emit_str("\x1b[32m[OK]\x1b[0m ");
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_nl();
    }};
}

#[cfg(feature = "no_logs")]
#[macro_export]
macro_rules! kok {
    ($($t:tt)*) => {{}};
}

/// kfail! - Log de falha (prefixo vermelho [FAIL]).
#[cfg(not(feature = "no_logs"))]
#[macro_export]
macro_rules! kfail {
    ($msg:expr) => {{
        $crate::drivers::serial::emit_str("\x1b[1;31m[FAIL]\x1b[0m ");
        $crate::drivers::serial::emit_str($msg);
        $crate::drivers::serial::emit_nl();
    }};
}

#[cfg(feature = "no_logs")]
#[macro_export]
macro_rules! kfail {
    ($($t:tt)*) => {{}};
}
