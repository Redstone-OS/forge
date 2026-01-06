//! # IPI - Inter-Processor Interrupts
//!
//! Envio de interrupções entre CPUs para coordenação.
//!
//! ## Vetores de IPI
//!
//! ```text
//! ┌───────┬────────────────────────────────────────┐
//! │ Vetor │ Propósito                              │
//! ├───────┼────────────────────────────────────────┤
//! │ 0xFE  │ Panic - Para todas as CPUs             │
//! │ 0xFD  │ TLB Invalidate - Shootdown             │
//! │ 0xFC  │ Reschedule - Força scheduler           │
//! │ 0xFB  │ Call Function - RPC remoto             │
//! └───────┴────────────────────────────────────────┘
//! ```

use super::topology;
use crate::arch::x86_64::apic::lapic;

/// Destino da IPI
#[derive(Debug, Clone, Copy)]
pub enum IpiTarget {
    /// Uma CPU específica (pelo APIC ID)
    Single(u32),
    /// Todas as CPUs (Broadcast)
    All,
    /// Todas exceto a atual
    AllButSelf,
}

/// Vetores de IPI reservados pelo kernel
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum IpiVector {
    /// Panic/Stop: Para todas as CPUs imediatamente
    Panic = 0xFE,
    /// TLB Shootdown: Invalida páginas
    TlbInvalidate = 0xFD,
    /// Reschedule: Força o scheduler a rodar
    Reschedule = 0xFC,
    /// Call Function: Executa função remota
    CallFunction = 0xFB,
}

/// Envia uma IPI para o destino especificado.
///
/// # Safety
///
/// Os handlers para os vetores devem estar instalados na IDT.
pub fn send_ipi(target: IpiTarget, vector: IpiVector) {
    let topo = topology::get();

    match target {
        IpiTarget::Single(apic_id) => unsafe {
            lapic::send_ipi(apic_id, vector as u8);
        },
        IpiTarget::All => {
            // Enviar para todas as CPUs
            for cpu in topo.iter() {
                unsafe {
                    lapic::send_ipi(cpu.apic_id, vector as u8);
                }
            }
        }
        IpiTarget::AllButSelf => {
            let my_id = lapic::id();
            for cpu in topo.iter() {
                if cpu.apic_id != my_id {
                    unsafe {
                        lapic::send_ipi(cpu.apic_id, vector as u8);
                    }
                }
            }
        }
    }
}

/// Envia IPI de reschedule para uma CPU específica.
#[inline]
pub fn send_reschedule(apic_id: u32) {
    unsafe {
        lapic::send_ipi(apic_id, IpiVector::Reschedule as u8);
    }
}

/// Envia IPI de TLB invalidate para todas as outras CPUs.
#[inline]
pub fn send_tlb_shootdown() {
    send_ipi(IpiTarget::AllButSelf, IpiVector::TlbInvalidate);
}

/// Envia IPI de panic para todas as CPUs (para em emergência).
#[inline]
pub fn send_panic_all() {
    send_ipi(IpiTarget::All, IpiVector::Panic);
}
