/// Arquivo: arch/_traits/cpu.rs
/// 
/// Propósito: Definição do trait abstrato `CpuTrait`.
/// Este trait estabelece o contrato que todas as implementações de arquitetura (x86_64, aarch64, etc.)
/// devem cumprir. Permite que o kernel opere de forma agnóstica à arquitetura em níveis superiores.
///
/// Detalhes de Implementação:
/// - Define métodos para controle de interrupções (cli/sti).
/// - Define método para halt da CPU (economia de energia).
/// - Define métodos para identificação do core atual.

//! Trait abstrato para operações de CPU

/// Trait que toda arquitetura deve implementar
pub trait CpuTrait {
    /// Desabilita interrupções (ex: cli em x86)
    fn disable_interrupts();
    
    /// Habilita interrupções (ex: sti em x86)
    fn enable_interrupts();
    
    /// Para a CPU até próxima interrupção (hlt)
    fn halt();
    
    /// Retorna ID do core atual
    fn current_core_id() -> u32;
    
    /// Retorna se interrupções estão habilitadas
    fn interrupts_enabled() -> bool;
}
