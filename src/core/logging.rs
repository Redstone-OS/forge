//! Sistema de Logging Unificado.
//!
//! Permite o uso de macros `kinfo!`, `kwarn!`, `kerror!` em qualquer lugar do kernel.

use crate::drivers::serial::SERIAL1;
use crate::sync::Mutex;
use core::fmt;

// TODO: Adicionar suporte ao Console quando ele estiver inicializado.
// Por enquanto, logs vão apenas para a Serial para garantir estabilidade.

/// Logger global.
pub struct KernelLogger;

impl KernelLogger {
    pub fn print_fmt(args: fmt::Arguments) {
        // Tenta adquirir o lock da serial e escrever.
        // Desabilita interrupções para evitar deadlocks se um IRQ tentar logar.
        use crate::arch::platform::cpu::X64Cpu; // Ou generic CpuOps
        use crate::arch::traits::CpuOps;

        let irq_enabled = X64Cpu::are_interrupts_enabled();
        if irq_enabled {
            X64Cpu::disable_interrupts();
        }

        // Bloco crítico
        {
            let mut serial = SERIAL1.lock();
            let _ = fmt::write(&mut *serial, args);
        }

        if irq_enabled {
            X64Cpu::enable_interrupts();
        }
    }
}

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
macro_rules! kinfo {
    ($($arg:tt)*) => ($crate::kprintln!("[INFO]  {}", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kwarn {
    ($($arg:tt)*) => ($crate::kprintln!("[WARN]  {}", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kerror {
    ($($arg:tt)*) => ($crate::kprintln!("[ERROR] {}", format_args!($($arg)*)));
}
