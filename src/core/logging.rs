//! Sistema de Logging do Kernel — Redstone OS
//! =========================================
//!
//! Logger profissional com filtragem por nível, cores ANSI e IRQ-safe.
//!
//! # Níveis de Log
//! - `ERROR`: Erros críticos (sempre visíveis)
//! - `WARN`: Situações suspeitas
//! - `INFO`: Fluxo normal de execução
//! - `DEBUG`: Informações de debug
//! - `TRACE`: Detalhes extremos (debug profundo)
//!
//! # Segurança
//! - Desabilita interrupções durante escrita
//! - Usa try_lock para evitar deadlocks
//! - Zero alocações durante log

use crate::arch::platform::Cpu;
use crate::arch::traits::CpuOps;
use crate::drivers::serial::SERIAL1;
use core::fmt;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};

/// Níveis de log - valores menores = mais críticos
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl LogLevel {
    /// Retorna o prefixo colorido ANSI para o nível
    #[inline]
    pub fn prefix(self) -> &'static str {
        match self {
            LogLevel::Error => "\x1b[1;31m[ERRO]\x1b[0m", // Vermelho bold
            LogLevel::Warn => "\x1b[1;33m[WARN]\x1b[0m",  // Amarelo bold
            LogLevel::Info => "\x1b[32m[INFO]\x1b[0m",    // Verde
            LogLevel::Debug => "\x1b[36m[DEBG]\x1b[0m",   // Ciano
            LogLevel::Trace => "\x1b[35m[TRAC]\x1b[0m",   // Magenta
        }
    }

    /// Retorna o prefixo sem cores
    #[inline]
    pub fn prefix_plain(self) -> &'static str {
        match self {
            LogLevel::Error => "[ERRO]",
            LogLevel::Warn => "[WARN]",
            LogLevel::Info => "[INFO]",
            LogLevel::Debug => "[DEBG]",
            LogLevel::Trace => "[TRAC]",
        }
    }

    /// Símbolo de status para console visual
    #[inline]
    pub fn symbol(self) -> &'static str {
        match self {
            LogLevel::Error => "✗",
            LogLevel::Warn => "⚠",
            LogLevel::Info => "•",
            LogLevel::Debug => "→",
            LogLevel::Trace => "·",
        }
    }
}

// Configuração global
// - Feature 'verbose_logs' (Cargo.toml): nível Trace (todos os logs)
// - Sem feature: nível Info (apenas info, warn, error)
#[cfg(feature = "verbose_logs")]
static GLOBAL_LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Trace as u8);

#[cfg(not(feature = "verbose_logs"))]
static GLOBAL_LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Info as u8);

static COLORS_ENABLED: AtomicBool = AtomicBool::new(true);
static LOG_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Logger global do Kernel.
pub struct KernelLogger;

impl KernelLogger {
    /// Log com nível específico
    pub fn log(level: LogLevel, args: fmt::Arguments) {
        // Filtrar por nível
        let current = GLOBAL_LOG_LEVEL.load(Ordering::Relaxed);
        if (level as u8) > current {
            return;
        }

        // Incrementar contador de mensagens
        let _msg_id = LOG_COUNTER.fetch_add(1, Ordering::Relaxed);

        // Entrar em seção crítica (IRQ-safe)
        let irq_enabled = Cpu::are_interrupts_enabled();
        if irq_enabled {
            unsafe {
                Cpu::disable_interrupts();
            }
        }

        // Escrever na Serial
        if let Some(mut serial) = SERIAL1.try_lock() {
            let prefix = if COLORS_ENABLED.load(Ordering::Relaxed) {
                level.prefix()
            } else {
                level.prefix_plain()
            };

            let _ = fmt::write(&mut *serial, format_args!("{} ", prefix));
            let _ = fmt::write(&mut *serial, args);
            let _ = fmt::write(&mut *serial, format_args!("\n"));
        }

        // Escrever no Console de vídeo
        crate::drivers::console::console_print_fmt(format_args!(
            "{} ",
            if COLORS_ENABLED.load(Ordering::Relaxed) {
                level.prefix()
            } else {
                level.prefix_plain()
            }
        ));
        crate::drivers::console::console_print_fmt(args);
        crate::drivers::console::console_print_fmt(format_args!("\n"));

        // Restaurar interrupções
        if irq_enabled {
            unsafe {
                Cpu::enable_interrupts();
            }
        }
    }

    /// Log raw sem prefixo (para kprint/kprintln)
    pub fn print_fmt(args: fmt::Arguments) {
        let irq_enabled = Cpu::are_interrupts_enabled();
        if irq_enabled {
            unsafe {
                Cpu::disable_interrupts();
            }
        }

        if let Some(mut serial) = SERIAL1.try_lock() {
            let _ = fmt::write(&mut *serial, args);
        }

        crate::drivers::console::console_print_fmt(args);

        if irq_enabled {
            unsafe {
                Cpu::enable_interrupts();
            }
        }
    }

    /// Define nível mínimo de log
    pub fn set_level(level: LogLevel) {
        GLOBAL_LOG_LEVEL.store(level as u8, Ordering::Relaxed);
    }

    /// Retorna o nível atual de log
    pub fn get_level() -> LogLevel {
        match GLOBAL_LOG_LEVEL.load(Ordering::Relaxed) {
            0 => LogLevel::Error,
            1 => LogLevel::Warn,
            2 => LogLevel::Info,
            3 => LogLevel::Debug,
            _ => LogLevel::Trace,
        }
    }

    /// Habilita/desabilita cores ANSI
    pub fn enable_colors(enabled: bool) {
        COLORS_ENABLED.store(enabled, Ordering::Relaxed);
    }

    /// Retorna o contador de mensagens de log
    pub fn message_count() -> u64 {
        LOG_COUNTER.load(Ordering::Relaxed)
    }
}

// === MACROS ===

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::core::logging::KernelLogger::print_fmt(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kerror {
    ($($arg:tt)*) => ($crate::core::logging::KernelLogger::log(
        $crate::core::logging::LogLevel::Error,
        format_args!($($arg)*)
    ));
}

#[macro_export]
macro_rules! kwarn {
    ($($arg:tt)*) => ($crate::core::logging::KernelLogger::log(
        $crate::core::logging::LogLevel::Warn,
        format_args!($($arg)*)
    ));
}

#[macro_export]
macro_rules! kinfo {
    ($($arg:tt)*) => ($crate::core::logging::KernelLogger::log(
        $crate::core::logging::LogLevel::Info,
        format_args!($($arg)*)
    ));
}

#[macro_export]
macro_rules! kdebug {
    ($($arg:tt)*) => ($crate::core::logging::KernelLogger::log(
        $crate::core::logging::LogLevel::Debug,
        format_args!($($arg)*)
    ));
}

#[macro_export]
macro_rules! ktrace {
    ($($arg:tt)*) => ($crate::core::logging::KernelLogger::log(
        $crate::core::logging::LogLevel::Trace,
        format_args!($($arg)*)
    ));
}

/// Macro para log OK (verde) - para status de inicialização
#[macro_export]
macro_rules! kok {
    ($($arg:tt)*) => {{
        $crate::kprint!("\x1b[32m[OK]\x1b[0m ");
        $crate::kprintln!($($arg)*);
    }};
}

/// Macro para log FAIL (vermelho) - para status de falha
#[macro_export]
macro_rules! kfail {
    ($($arg:tt)*) => {{
        $crate::kprint!("\x1b[1;31m[FAIL]\x1b[0m ");
        $crate::kprintln!($($arg)*);
    }};
}
