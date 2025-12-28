/// Arquivo: core/boot/panic.rs
///
/// Propósito: Handler de Pânico do Kernel.
/// Esta função é chamada automaticamente pelo Rust quando ocorre um `panic!()`.
/// É o ponto final de falhas irrecuperáveis.
///
/// Detalhes de Implementação:
/// - Imprime mensagem de erro e localização.
/// - Desabilita interrupções.
/// - Trava a CPU (loop infinito com HLT).
/// - (Futuro) Parar outras CPUs via IPI.
/// - (Futuro) Dump de stack trace.
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Desabilita interrupções imediatamente para evitar reentrância ou ruído
    unsafe {
        crate::arch::Cpu::disable_interrupts();
    }

    crate::kerror!("*****************************************************");
    crate::kerror!("*                   PANICO DO KERNEL                    *");
    crate::kerror!("*****************************************************");

    if let Some(location) = info.location() {
        crate::kerror!("Arquivo:", 0u64); // TODO: suporte a str
        crate::kerror!(location.file());
        crate::kerror!("Linha:", location.line() as u64);
    } else {
        crate::kerror!("Localização: Desconhecida");
    }

    // Tenta extrair a mensagem de pânico
    crate::kerror!("Mensagem:", 0u64); // TODO: suporte a str
    let payload = info.payload();
    if let Some(s) = payload.downcast_ref::<&str>() {
        crate::kerror!(s);
    } else {
        crate::kerror!("(Erro desconhecido ou não-string)");
    }

    crate::kerror!("*****************************************************");
    crate::kerror!("*             SISTEMA HALTED FOREVER                *");
    crate::kerror!("*****************************************************");

    // TODO: Enviar IPI para parar outras CPUs (crate::smp::ipi::send_context(Panic))

    loop {
        unsafe {
            crate::arch::Cpu::halt();
        }
    }
}
