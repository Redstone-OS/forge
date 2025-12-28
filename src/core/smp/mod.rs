/// Arquivo: core/smp/mod.rs
///
/// Propósito: Módulo de Multiprocessamento Simétrico (SMP).
/// Gerencia a descoberta, inicialização e comunicação entre múltiplos cores de CPU.
///
/// Módulos contidos:
/// - `percpu`: Variáveis locais de CPU.
/// - `topology`: Detecção de Cores/Sockets.
/// - `bringup`: Inicialização de APs (Application Processors).
/// - `ipi`: Inter-Processor Interrupts.

pub mod percpu;
pub mod topology;
pub mod bringup;
pub mod ipi;
