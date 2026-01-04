//! # ACPI Table Definitions
//!
//! Definições de estruturas para tabelas ACPI comuns.

use super::AcpiTableHeader;

// =============================================================================
// SIGNATURES DE TABELAS
// =============================================================================

/// RSDT - Root System Description Table
pub const RSDT_SIGNATURE: [u8; 4] = *b"RSDT";

/// XSDT - Extended System Description Table
pub const XSDT_SIGNATURE: [u8; 4] = *b"XSDT";

/// MADT - Multiple APIC Description Table
pub const MADT_SIGNATURE: [u8; 4] = *b"APIC";

/// FADT - Fixed ACPI Description Table (também chamada FACP)
pub const FADT_SIGNATURE: [u8; 4] = *b"FACP";

/// MCFG - PCI Express Memory-Mapped Configuration Space
pub const MCFG_SIGNATURE: [u8; 4] = *b"MCFG";

/// HPET - High Precision Event Timer
pub const HPET_SIGNATURE: [u8; 4] = *b"HPET";

/// DSDT - Differentiated System Description Table
pub const DSDT_SIGNATURE: [u8; 4] = *b"DSDT";

/// SSDT - Secondary System Description Table
pub const SSDT_SIGNATURE: [u8; 4] = *b"SSDT";

/// BGRT - Boot Graphics Resource Table
pub const BGRT_SIGNATURE: [u8; 4] = *b"BGRT";

// =============================================================================
// MCFG - PCI EXPRESS CONFIGURATION
// =============================================================================

/// MCFG Entry - cada entrada descreve um segmento PCIe
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct McfgEntry {
    /// Endereço base do ECAM
    pub base_address: u64,
    /// Segmento PCI
    pub segment: u16,
    /// Primeiro barramento neste segmento
    pub start_bus: u8,
    /// Último barramento neste segmento
    pub end_bus: u8,
    /// Reservado
    pub reserved: u32,
}

// =============================================================================
// HPET - HIGH PRECISION EVENT TIMER
// =============================================================================

/// HPET Table
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct HpetTable {
    pub header: AcpiTableHeader,
    /// Hardware revision ID
    pub hardware_rev_id: u8,
    /// Número de comparadores no bloco
    pub comparator_count: u8,
    /// Número do timer
    pub timer_number: u16,
    /// Frequência mínima de clock
    pub min_clock_tick: u16,
    /// Atributos de página
    pub page_protection: u8,
    /// Reservado
    pub reserved: u8,
    /// Endereço base do HPET
    pub base_address: HpetAddress,
    /// HPET number
    pub hpet_number: u8,
    /// Tick mínimo do main counter
    pub main_counter_min_tick: u16,
    /// Atributos de proteção
    pub page_protection_oem: u8,
}

/// Endereço do HPET (Generic Address Structure)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct HpetAddress {
    pub address_space_id: u8,
    pub register_bit_width: u8,
    pub register_bit_offset: u8,
    pub access_size: u8,
    pub address: u64,
}

// =============================================================================
// GENERIC ADDRESS STRUCTURE (GAS)
// =============================================================================

/// Generic Address Structure - usado em várias tabelas
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GenericAddress {
    /// Address space ID (0=system memory, 1=system I/O)
    pub address_space_id: u8,
    /// Largura do registrador em bits
    pub register_bit_width: u8,
    /// Offset do bit dentro do registrador
    pub register_bit_offset: u8,
    /// Tamanho do acesso (0=undefined, 1=byte, 2=word, 3=dword, 4=qword)
    pub access_size: u8,
    /// Endereço
    pub address: u64,
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Verifica signature de uma tabela.
pub fn check_signature(header: &AcpiTableHeader, expected: &[u8; 4]) -> bool {
    &header.signature == expected
}

/// Valida checksum de uma tabela ACPI.
///
/// A soma de todos os bytes deve ser 0 (mod 256).
pub fn validate_checksum(data: &[u8]) -> bool {
    let sum: u8 = data.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
    sum == 0
}
