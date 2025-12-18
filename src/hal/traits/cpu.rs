//! Trait de CPU
//!
//! # TODOs
//! - TODO(prioridade=alta, versão=v1.0): Definir trait completo

/// Abstração de CPU
pub trait CpuHal {
    /// Retorna ID da CPU atual
    fn id() -> usize;
    
    /// Para a CPU (halt)
    fn halt();
    
    /// Habilita interrupções
    fn enable_interrupts();
    
    /// Desabilita interrupções
    fn disable_interrupts();
}
