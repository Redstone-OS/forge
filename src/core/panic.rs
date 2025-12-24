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
use core::panic::PanicInfo;

/// Handler chamado pelo compilador Rust em `panic!`.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 1. Silence: Desabilitar interrupções imediatamente para evitar
    // que o scheduler ou drivers tentem rodar em estado corrompido.
    // SAFETY: Estamos em shutdown de emergência. É a ação mais segura.
    unsafe {
        Cpu::disable_interrupts();
    }

    // 2. Report: Tentar extrair informações úteis
    crate::kerror!("=====   PÂNICO DO KERNEL (CRÍTICO)   =====");

    if let Some(location) = info.location() {
        crate::kerror!(
            "Localização: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        crate::kerror!("Localização: Desconhecida (Sem debug info)");
    }

    if let Some(msg) = info.message() {
        crate::kerror!("Razão:   {}", msg);
    } else {
        crate::kerror!("Razão:   Causa desconhecida");
    }

    crate::kerror!("Sistema congelado. Reset manual necessário.");

    // 3. Freeze: Entra em loop infinito de HLT.
    // O método hang() já garante o loop e desabilita interrupções novamente por segurança.
    Cpu::hang();
}
