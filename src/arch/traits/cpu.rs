//! Interface Abstrata de CPU (HAL).
//! Define as operações que qualquer arquitetura (x86, ARM, RISC-V) deve implementar.

pub trait CpuOps {
    /// Para a execução da CPU até a próxima interrupção (instrução HLT).
    /// Economiza energia em loops ociosos.
    fn halt();

    /// Desabilita interrupções globalmente (CLI).
    /// Crítico para seções atômicas no kernel.
    fn disable_interrupts();

    /// Habilita interrupções globalmente (STI).
    fn enable_interrupts();

    /// Verifica se as interrupções estão habilitadas.
    fn are_interrupts_enabled() -> bool;

    /// Entra em loop infinito de halt com interrupções desabilitadas.
    /// Usado em pânicos irrecuperáveis.
    fn hang() -> ! {
        Self::disable_interrupts();
        loop {
            Self::halt();
        }
    }
}
