/// Arquivo: core/debug/kdebug.rs
///
/// Propósito: Ferramentas de depuração de baixo nível.
/// Permite invocar breakpoints programáticos e asserções de tempo de execução
/// que se integram com o sistema de logs.
///
/// Detalhes de Implementação:
/// - Como o módulo `core` não pode usar assembly, o `breakpoint` aqui
///   apenas simula uma parada crítica ou chama uma abstração se disponível.
/// - Por enquanto, usamos um loop infinito com log para "parar" o kernel.

// Depurador do Kernel (KDebug)

/// Dispara um breakpoint de software (parada de execução).
///
/// Útil para parar o kernel e inspecionar estado via debugger (GDB/QEMU)
/// ou simplesmente travar a execução para análise de log.
pub fn breakpoint() {
    crate::kwarn!("--- KERNEL BREAKPOINT ---");

    // Idealmente chamaríamos algo como crate::arch::Cpu::breakpoint();
    // Como não temos isso no trait ainda, vamos apenas desabilitar interrupções e travar.
    crate::arch::Cpu::disable_interrupts();
    loop {
        crate::arch::Cpu::halt();
    }
}

/// Função auxiliar chamada por macros de assert (se implementarmos customizadas).
pub fn assert_failed(expr: &str, file: &str, line: u32) -> ! {
    crate::kerror!("FALHA DE ASSERÇÃO:");
    crate::kerror!("Expr:", 0); // TODO: Passar str
                                // Como kerror! com string variável é limitado pelo macro atual, simplificamos:
    crate::kerror!(expr);
    crate::kerror!("Arquivo:", 0); // Placeholder
    crate::kerror!(file);

    panic!("Assertion failed at {}:{}", file, line);
}
