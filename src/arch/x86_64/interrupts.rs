/// Arquivo: x86_64/interrupts.rs
///
/// Propósito: Definição e registro de Handlers de Interrupção.
/// Configura a IDT com handlers para exceções da CPU (Divide Error, Page Fault, GPF, etc.)
/// e mapeia IRQs de hardware.
///
/// Detalhes de Implementação:
/// - Inicializa a IDT global.
/// - Define stubs/handlers para as exceções críticas.
/// - Implementa `init_idt`.

//! Handlers de interrupção e inicialização da IDT

use crate::arch::x86_64::idt::IDT;

/// Inicializa a Tabela de Descritores de Interrupção (IDT).
/// Deve ser chamado no boot antes de habilitar interrupções.
pub fn init_idt() {
    // SAFETY: Acesso à estática mutável IDT é seguro aqui pois estamos no boot single-core
    unsafe {
        // Registrar handlers de exceção básicos
        IDT.set_handler(0, exception_handler_stub as u64); // Divide Error
        IDT.set_handler(3, exception_handler_stub as u64); // Breakpoint
        IDT.set_handler(6, exception_handler_stub as u64); // Invalid Opcode
        IDT.set_handler(8, exception_handler_stub as u64); // Double Fault
        IDT.set_handler(13, exception_handler_stub as u64); // GPF
        IDT.set_handler(14, exception_handler_stub as u64); // Page Fault
        
        // Carregar IDT
        IDT.load();
    }
}

/// Stub temporário para capturar exceções.
/// Em uma implementação real, usariamos `naked functions` ou trampolins em assembly
/// para salvar `TrapFrame` antes de chamar Rust.
/// 
/// # Safety
/// 
/// Esta função é chamada diretamente pelo hardware (via IDT entry).
/// Como está declarada `extern "C"`, o compilador gera um prólogo normal, 
/// mas interrupções empilham [SS, RSP, RFLAGS, CS, RIP] (e Error Code as vezes).
/// Um handler real precisa usar `iretq`.
/// 
/// PARA EVITAR CRASH IMEDIATO: Isso é apenas um placeholder.
/// O sistema vai crashar se uma exceção ocorrer porque não há `iretq`.
/// O objetivo aqui é ter o símbolo para compilar.
#[no_mangle]
extern "C" fn exception_handler_stub() {
    // TODO: Implementar handlers reais em assembly ou usar x86-interrupt calling convention
    loop {
        crate::arch::x86_64::cpu::Cpu::halt();
    }
}
