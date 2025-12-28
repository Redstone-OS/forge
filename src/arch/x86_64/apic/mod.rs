/// Arquivo: x86_64/apic/mod.rs
///
/// Propósito: Módulo de gerenciamento do Advanced Programmable Interrupt Controller (APIC).
/// O APIC é responsável por rotear interrupções em sistemas x86 modernos (substituindo o PIC 8259).
/// Divide-se em:
/// - Local APIC (LAPIC): Um por core, gerencia interrupções locais e Timer.
/// - I/O APIC: Global, roteia interrupções de hardware externo para os LAPICs.
///
/// Módulos contidos:
/// - `lapic`: Controlador Local (dentro da CPU).
/// - `ioapic`: Controlador de I/O (no chipset).

pub mod lapic;
pub mod ioapic;
