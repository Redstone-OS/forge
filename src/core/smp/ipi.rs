/// Arquivo: core/smp/ipi.rs
///
/// Propósito: Envio de Interrupções Inter-Processador (IPIs).
/// Usado para coordenar atividades em sistemas SMP, como TLB Shootdown,
/// parada de emergência (panic) ou acordar CPUs.
///
/// Detalhes de Implementação:
/// - Abstrai o envio de IPIs para a camada de arquitetura.
/// - Define tipos de alvos (Target).

//! Inter-Processor Interrupts

use super::topology::CpuId;

/// Destino da IPI
#[derive(Debug, Clone, Copy)]
pub enum IpiTarget {
    /// Uma CPU específica
    Single(CpuId),
    /// Todas as CPUs (Broadcast)
    All,
    /// Todas exceto a atual
    AllButSelf,
}

/// Vetores de IPI comuns (definidos por convenção no kernel)
/// TODO: Mover para um header compartilhado de interrupções
#[repr(u8)]
pub enum IpiVector {
    /// Panic/Stop: Para todas as CPUs imediatamente
    Panic = 0xFE,
    /// TLB Shootdown: Invalida páginas
    TlbInvalidate = 0xFD,
    /// Reschedule: Força o scheduler a rodar
    Reschedule = 0xFC,
    /// Call Function: Executa função remota (generic)
    CallFunction = 0xFB,
}

/// Envia uma IPI para o destino especificado.
pub fn send_ipi(target: IpiTarget, vector: IpiVector) {
    // TODO: Chamar implementação da arquitetura.
    // Como não temos isso exposto no trait CpuTrait ainda, acessamos via hack
    // ou deixamos o TODO. O ideal seria crate::arch::send_ipi(...)
    
    match target {
        IpiTarget::Single(id) => {
             // crate::arch::apic::send_ipi(id, vector as u8);
             let _ = id;
        }
        _ => {
            // Broadcast
        }
    }
    
    // Placeholder para evitar unused variable warning
    let _ = vector;
    
    // crate::kdebug!("IPI enviada (simulada)");
}
