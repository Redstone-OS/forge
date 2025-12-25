//! Sistema de Logging Unificado.
//!
//! Este módulo fornece a infraestrutura de saída de texto para o kernel.
//! É projetado para ser "Best Effort" e seguro contra reentrância e deadlocks
//! em contextos de interrupção.
//!
//! # Safety
//! O logger desabilita interrupções temporariamente para garantir a atomicidade
//! da escrita na porta serial e evitar deadlocks com handlers de IRQ.

use crate::arch::platform::Cpu;
use crate::arch::traits::CpuOps;
use crate::drivers::serial::SERIAL1;
use core::fmt;

/// Logger global do Kernel.
/// Não armazena estado interno, apenas orquestra o acesso aos drivers.
pub struct KernelLogger;

impl KernelLogger {
    /// Ponto de entrada para as macros `kprint!`, etc.
    ///
    /// # Arquitetura de Segurança
    /// 1. Salva o estado atual das interrupções.
    /// 2. Desabilita interrupções (CLI) para entrar em seção crítica.
    /// 3. Tenta adquirir o lock da Serial.
    /// 4. Escreve.
    /// 5. Restaura interrupções se estavam habilitadas.
    pub fn print_fmt(args: fmt::Arguments) {
        // 1. Verificar estado de interrupção
        let irq_enabled = Cpu::are_interrupts_enabled();

        // 2. Entrar em seção crítica (No-Interrupts zone)
        // SAFETY: Necessário para evitar que uma interrupção ocorra enquanto seguramos
        // o lock da serial, o que causaria deadlock se a interrupção também tentasse logar.
        if irq_enabled {
            unsafe {
                Cpu::disable_interrupts();
            }
        }

        // 3. Escrita Atômica (Serial + Video)
        {
            // Usamos try_lock para evitar travar o kernel se a serial estiver corrompida
            // ou permanentemente travada por um panic anterior.
            if let Some(mut serial) = SERIAL1.try_lock() {
                let _ = fmt::write(&mut *serial, args);
            }

            // Escrever no Console de Vídeo (se disponível)
            // O Console tem seu próprio locking interno no método helper
            crate::drivers::console::console_print_fmt(args);
        }

        // 4. Sair da seção crítica
        // SAFETY: Restauramos o estado anterior. Se estava habilitado, reabilitamos.
        if irq_enabled {
            unsafe {
                Cpu::enable_interrupts();
            }
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

// Log Levels com formatação visual consistente
#[macro_export]
macro_rules! kinfo {
    ($($arg:tt)*) => ($crate::kprintln!("\x1b[32m[INFO]\x1b[0m {}", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kwarn {
    ($($arg:tt)*) => ($crate::kprintln!("\x1b[33m[WARN]\x1b[0m {}", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kerror {
    ($($arg:tt)*) => ($crate::kprintln!("\x1b[31m[ERRO]\x1b[0m {}", format_args!($($arg)*)));
}
