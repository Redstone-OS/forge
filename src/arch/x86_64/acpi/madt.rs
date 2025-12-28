/// Arquivo: x86_64/acpi/madt.rs
///
/// Propósito: Parsing da Multiple APIC Description Table (MADT).
/// Esta tabela descreve todos os controladores de interrupção (Local APICs e I/O APICs)
/// presentes no sistema. É fundamental para iniciar Multi-Processing (SMP).
///
/// Detalhes de Implementação:
/// - Define a estrutura do Header da MADT.
/// - Define estruturas para as entradas variáveis (Records) que seguem o header.
/// - Tipos comuns:
///   - Tipo 0: Processor Local APIC.
///   - Tipo 1: I/O APIC.
///   - Tipo 2: Interrupt Source Override (ISO) - Crucial para teclados e timers legacy.

/// ACPI MADT (Multiple APIC Description Table)

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MadtHeader {
    pub signature: [u8; 4], // "APIC"
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,

    // Campos específicos MADT
    pub local_apic_address: u32, // Endereço físico base dos Local APICs
    pub flags: u32,              // Bit 0 = PCAT_COMPAT (tem PIC 8259?)
}

/// Cabeçalho genérico para registros da MADT
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MadtEntryHeader {
    pub entry_type: u8,
    pub record_length: u8,
}

/// Tipo 0: Processor Local APIC
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MadtLocalApic {
    pub header: MadtEntryHeader,
    pub acpi_processor_id: u8,
    pub apic_id: u8,
    pub flags: u32, // Bit 0 = Processor Enumbled
}

/// Tipo 1: I/O APIC
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MadtIoApic {
    pub header: MadtEntryHeader,
    pub io_apic_id: u8,
    pub reserved: u8,
    pub io_apic_address: u32,
    pub global_system_interrupt_base: u32,
}

/// Tipo 2: Interrupt Source Override (ISO)
/// Usado para mapear IRQs ISA (ex: 0 para Timer, 1 para Teclado) para GSI.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MadtIso {
    pub header: MadtEntryHeader,
    pub bus_source: u8, // Sempre 0 (ISA)
    pub irq_source: u8, // IRQ no PIC (ex: 1 para teclado)
    pub gsi: u32,       // Global System Interrupt no I/O APIC (ex: 1)
    pub flags: u16,     // Polarity/Trigger Mode
}

/// Tipo 4: Local APIC NMI
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MadtNmi {
    pub header: MadtEntryHeader,
    pub acpi_processor_id: u8, // 0xFF = All processors
    pub flags: u16,
    pub lint: u8, // LINT# input (0 ou 1)
}

// TODO: Implementar iterador seguro sobre as entradas da MADT
// (Requer apenas ler bytes raw a partir de &MadtHeader + sizeof(MadtHeader))
