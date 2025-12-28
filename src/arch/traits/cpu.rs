//! Trait abstrato para operações de CPU

/// Trait que toda arquitetura deve implementar
pub trait CpuTrait {
    /// Desabilita interrupções (cli)
    fn disable_interrupts();
    
    /// Habilita interrupções (sti)
    fn enable_interrupts();
    
    /// Para a CPU até próxima interrupção (hlt)
    fn halt();
    
    /// Retorna ID do core atual
    fn current_core_id() -> u32;
    
    /// Retorna se interrupções estão habilitadas
    fn interrupts_enabled() -> bool;
}
