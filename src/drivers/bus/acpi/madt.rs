//! # MADT Parser (Multiple APIC Description Table)
//!
//! Este arquivo parseia a tabela **MADT** (também chamada APIC table)
//! que descreve os controladores de interrupção do sistema.
//!
//! ## Informações Contidas:
//! - Local APICs (um por CPU lógica)
//! - I/O APICs (controladores de interrupção externa)
//! - Overrides de interrupção (ex: IRQ 0 mapeada para APIC vector 2)
//! - NMI sources
//!
//! ## STUB:
//! Estruturas definidas, parsing não implementado.

use super::AcpiTableHeader;

// =============================================================================
// ESTRUTURA DA MADT
// =============================================================================

/// Header da MADT
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MadtHeader {
    pub header: AcpiTableHeader,
    /// Endereço base do Local APIC
    pub local_apic_address: u32,
    /// Flags
    pub flags: u32,
    // Seguido por entradas de tamanho variável
}

/// Flags da MADT
pub const MADT_FLAG_PCAT_COMPAT: u32 = 0x01; // Dual 8259 PICs disponíveis

// =============================================================================
// TIPOS DE ENTRADA DA MADT
// =============================================================================

/// Tipo de entrada na MADT
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MadtEntryType {
    LocalApic = 0,
    IoApic = 1,
    InterruptOverride = 2,
    NmiSource = 3,
    LocalApicNmi = 4,
    LocalApicOverride = 5,
    IoSapic = 6,
    LocalSapic = 7,
    PlatformInterrupt = 8,
    LocalX2Apic = 9,
    LocalX2ApicNmi = 0x0A,
    Gicc = 0x0B, // ARM GIC CPU Interface
    Gicd = 0x0C, // ARM GIC Distributor
    GicMsiFrame = 0x0D,
    Gicr = 0x0E,   // ARM GIC Redistributor
    GicIts = 0x0F, // ARM GIC ITS
}

/// Header comum a todas as entradas MADT
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MadtEntryHeader {
    pub entry_type: u8,
    pub length: u8,
}

// =============================================================================
// TIPOS ESPECÍFICOS DE ENTRADA
// =============================================================================

/// Local APIC entry (Type 0)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MadtLocalApic {
    pub header: MadtEntryHeader,
    /// ACPI Processor ID
    pub acpi_processor_id: u8,
    /// APIC ID
    pub apic_id: u8,
    /// Flags (bit 0 = enabled)
    pub flags: u32,
}

/// I/O APIC entry (Type 1)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MadtIoApic {
    pub header: MadtEntryHeader,
    /// I/O APIC ID
    pub io_apic_id: u8,
    /// Reservado
    pub reserved: u8,
    /// Endereço base (MMIO)
    pub address: u32,
    /// Global System Interrupt base
    pub gsi_base: u32,
}

/// Interrupt Source Override (Type 2)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MadtInterruptOverride {
    pub header: MadtEntryHeader,
    /// Bus (sempre 0 = ISA)
    pub bus: u8,
    /// Source IRQ (IRQ legada)
    pub source: u8,
    /// Global System Interrupt (destino APIC)
    pub gsi: u32,
    /// Flags (polaridade e trigger mode)
    pub flags: u16,
}

/// Local APIC NMI (Type 4)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MadtLocalApicNmi {
    pub header: MadtEntryHeader,
    /// ACPI Processor ID (0xFF = all processors)
    pub acpi_processor_id: u8,
    /// Flags
    pub flags: u16,
    /// Local APIC LINT# (0 ou 1)
    pub lint: u8,
}

/// Local APIC Address Override (Type 5)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MadtLocalApicOverride {
    pub header: MadtEntryHeader,
    /// Reservado
    pub reserved: u16,
    /// Endereço 64-bit do Local APIC
    pub address: u64,
}

// =============================================================================
// RESULTADO DO PARSING
// =============================================================================

/// Informações extraídas da MADT.
#[derive(Debug, Clone, Default)]
pub struct MadtInfo {
    /// Endereço base do Local APIC
    pub local_apic_address: u64,

    /// Lista de Local APICs (CPUs)
    pub local_apics: alloc::vec::Vec<LocalApicInfo>,

    /// Lista de I/O APICs
    pub io_apics: alloc::vec::Vec<IoApicInfo>,

    /// Overrides de IRQ
    pub overrides: alloc::vec::Vec<IrqOverride>,

    /// Fontes de NMI
    pub nmis: alloc::vec::Vec<NmiInfo>,

    /// Flags
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub struct LocalApicInfo {
    pub processor_id: u8,
    pub apic_id: u8,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct IoApicInfo {
    pub id: u8,
    pub address: u32,
    pub gsi_base: u32,
}

#[derive(Debug, Clone)]
pub struct IrqOverride {
    pub source_irq: u8,
    pub gsi: u32,
    pub flags: u16,
}

#[derive(Debug, Clone)]
pub struct NmiInfo {
    pub processor_id: u8,
    pub lint: u8,
    pub flags: u16,
}

// =============================================================================
// FUNÇÕES DE PARSING
// =============================================================================

/// Parseia a tabela MADT.
///
/// ## STUB:
/// Não implementado. Retorna estrutura vazia.
pub fn parse(_madt_address: u64) -> MadtInfo {
    crate::kwarn!("(MADT) parse() não implementado");

    // TODO: Implementar
    // 1. Ler header
    // 2. Iterar por entradas
    // 3. Parsear cada tipo

    MadtInfo::default()
}

/// Retorna número de CPUs detectadas.
pub fn cpu_count(info: &MadtInfo) -> usize {
    info.local_apics.iter().filter(|a| a.enabled).count()
}
