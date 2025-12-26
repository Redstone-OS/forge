//! Panic Handler (Kernel Crash Reporter).
//!
//! O último recurso do sistema. Quando invocado, assume-se que o estado do kernel
//! é inconsistente e irrecuperável.
//!
//! # Protocolo de Pânico
//! 1. Desabilitar Interrupções (Imediato).
//! 2. Logar Causa e Localização.
//! 3. Halt Loop (Congelar CPU).

use crate::arch::platform::Cpu;
use crate::arch::traits::CpuOps;
use crate::drivers::serial;
use core::panic::PanicInfo;

/// Handler chamado pelo compilador Rust em `panic!`.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 1. Silêncio: Desabilitar interrupções imediatamente.
    unsafe {
        Cpu::disable_interrupts();
    }

    // 2. Reportar: Tentar extrair informações úteis via serial direta (sem core::fmt)
    serial::emit_str("\x1b[1;31m"); // Vermelho Bold
    serial::emit_str("\n\r\n\r=====   PANICO DO KERNEL (CRITICO)   =====");
    serial::emit_str("\x1b[0m\n\r");

    if let Some(location) = info.location() {
        serial::emit_str("[LOCAL] ");
        serial::emit_str(location.file());
        serial::emit_str(":");
        serial::emit_dec(location.line() as usize);
        serial::emit_str(":");
        serial::emit_dec(location.column() as usize);
        serial::emit_nl();
    } else {
        serial::emit_str("[LOCAL] Desconhecido\n\r");
    }

    // Nota: info.message() requer o sistema de formatação do Rust (core::fmt)
    // que estamos evitando para prevenir crashes #UD por SSE/AVX.
    serial::emit_str("[INFO] Veja a ultima linha de log para mais detalhes.\n\r");
    serial::emit_str("[HALT] Sistema congelado. Reset manual necessario.\n\r");

    // 3. Congelar: Entra em loop infinito de HLT.
    Cpu::hang();
}
