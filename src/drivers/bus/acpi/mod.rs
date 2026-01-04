//! # ACPI - Advanced Configuration and Power Interface
//!
//! Este módulo implementa o parser de tabelas **ACPI** - o padrão moderno
//! para descoberta de hardware e gerenciamento de energia em PCs.
//!
//! ## Tabelas ACPI Importantes:
//! - **RSDP**: Root System Description Pointer (ponto de entrada)
//! - **RSDT/XSDT**: Root/Extended System Description Table
//! - **MADT**: Multiple APIC Description Table (APICs, IOAPICs)
//! - **FADT**: Fixed ACPI Description Table (power, PM timer)
//! - **MCFG**: PCI Express Memory-Mapped Config Space
//! - **HPET**: High Precision Event Timer
//! - **DSDT/SSDT**: Differentiated/Secondary System Description Tables
//!
//! ## Fluxo de Inicialização:
//! 1. Localiza RSDP (via EFI config table ou busca em memória)
//! 2. Valida e parseia RSDT/XSDT
//! 3. Percorre tabelas filhas
//! 4. Extrai informações relevantes (APICs, ECAM base, etc)
//!
//! ## STUB:
//! Implementação parcial. Estruturas definidas mas parsing incompleto.

pub mod fadt;
pub mod madt; // Parser da MADT (APICs)
pub mod tables; // Definições de estruturas de tabelas // Parser da FADT (Power)

use crate::sync::Spinlock;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Signature do RSDP: "RSD PTR "
pub const RSDP_SIGNATURE: [u8; 8] = *b"RSD PTR ";

/// Região de memória onde buscar RSDP (EBDA + BIOS ROM)
pub const RSDP_SEARCH_START: u64 = 0x000E_0000;
pub const RSDP_SEARCH_END: u64 = 0x000F_FFFF;

// =============================================================================
// ESTRUTURAS PRINCIPAIS
// =============================================================================

/// Root System Description Pointer (RSDP) - versão 1.0
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct RsdpV1 {
    /// Signature: "RSD PTR "
    pub signature: [u8; 8],
    /// Checksum (soma de todos bytes = 0)
    pub checksum: u8,
    /// OEM ID
    pub oem_id: [u8; 6],
    /// Revisão (0 = ACPI 1.0, 2 = ACPI 2.0+)
    pub revision: u8,
    /// Endereço físico da RSDT
    pub rsdt_address: u32,
}

/// Root System Description Pointer (RSDP) - versão 2.0+
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct RsdpV2 {
    /// Campos da versão 1.0
    pub v1: RsdpV1,
    /// Tamanho total da estrutura
    pub length: u32,
    /// Endereço físico da XSDT (64 bits)
    pub xsdt_address: u64,
    /// Checksum estendido
    pub extended_checksum: u8,
    /// Reservado
    pub reserved: [u8; 3],
}

/// Header comum a todas as tabelas ACPI
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct AcpiTableHeader {
    /// Signature (4 bytes ASCII)
    pub signature: [u8; 4],
    /// Tamanho total da tabela
    pub length: u32,
    /// Revisão
    pub revision: u8,
    /// Checksum
    pub checksum: u8,
    /// OEM ID
    pub oem_id: [u8; 6],
    /// OEM Table ID
    pub oem_table_id: [u8; 8],
    /// OEM Revision
    pub oem_revision: u32,
    /// Creator ID
    pub creator_id: u32,
    /// Creator Revision
    pub creator_revision: u32,
}

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Informações ACPI descobertas.
static ACPI_INFO: Spinlock<AcpiInfo> = Spinlock::new(AcpiInfo {
    rsdp_address: 0,
    rsdt_address: 0,
    xsdt_address: 0,
    revision: 0,
    madt_address: 0,
    fadt_address: 0,
    mcfg_address: 0,
    hpet_address: 0,
    initialized: false,
});

/// Informações coletadas do ACPI.
#[derive(Debug, Clone)]
pub struct AcpiInfo {
    pub rsdp_address: u64,
    pub rsdt_address: u64,
    pub xsdt_address: u64,
    pub revision: u8,
    pub madt_address: u64,
    pub fadt_address: u64,
    pub mcfg_address: u64,
    pub hpet_address: u64,
    pub initialized: bool,
}

// =============================================================================
// FUNÇÕES DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa o subsistema ACPI.
pub fn init() {
    crate::kinfo!("(ACPI) Inicializando parser ACPI...");

    // Tenta localizar RSDP
    if let Some(rsdp_addr) = find_rsdp() {
        crate::kinfo!("(ACPI) RSDP encontrado em:", rsdp_addr);

        let mut info = ACPI_INFO.lock();
        info.rsdp_address = rsdp_addr;

        // Parseia RSDP
        if let Some(parsed) = parse_rsdp(rsdp_addr) {
            info.revision = parsed.revision;
            info.rsdt_address = parsed.rsdt_address;
            info.xsdt_address = parsed.xsdt_address;

            crate::kinfo!("(ACPI) Revisão:", parsed.revision);
            crate::kinfo!("(ACPI) RSDT:", parsed.rsdt_address);

            // Parseia tabelas
            if parsed.xsdt_address != 0 {
                parse_xsdt(&mut info, parsed.xsdt_address);
            } else if parsed.rsdt_address != 0 {
                parse_rsdt(&mut info, parsed.rsdt_address);
            }

            info.initialized = true;
        }
    } else {
        crate::kwarn!("(ACPI) RSDP não encontrado - ACPI indisponível");
    }

    crate::kinfo!("(ACPI) Inicialização completa");
}

/// Localiza o RSDP na memória.
///
/// ## STUB:
/// Deveria verificar EFI config table primeiro, depois buscar em memória.
fn find_rsdp() -> Option<u64> {
    crate::kwarn!("(ACPI) find_rsdp() stub - usando busca em memória");

    // TODO: Verificar EFI System Table primeiro (se disponível)

    // Busca em memória BIOS
    // Na prática, precisamos acessar essa memória mapeada
    // Por enquanto, retorna None

    None
}

/// Estrutura temporária para resultado do parse do RSDP.
struct ParsedRsdp {
    revision: u8,
    rsdt_address: u64,
    xsdt_address: u64,
}

/// Parseia o RSDP.
fn parse_rsdp(addr: u64) -> Option<ParsedRsdp> {
    crate::kwarn!("(ACPI) parse_rsdp() stub");

    // TODO: Ler estrutura da memória e validar checksum

    None
}

/// Parseia a RSDT (32 bits).
fn parse_rsdt(info: &mut AcpiInfo, addr: u64) {
    crate::kwarn!("(ACPI) parse_rsdt() stub");
    // TODO: Implementar
}

/// Parseia a XSDT (64 bits).
fn parse_xsdt(info: &mut AcpiInfo, addr: u64) {
    crate::kwarn!("(ACPI) parse_xsdt() stub");
    // TODO: Implementar
}

// =============================================================================
// FUNÇÕES PÚBLICAS DE CONSULTA
// =============================================================================

/// Retorna informações ACPI.
pub fn get_info() -> AcpiInfo {
    ACPI_INFO.lock().clone()
}

/// Retorna endereço da MADT (se disponível).
pub fn get_madt_address() -> Option<u64> {
    let info = ACPI_INFO.lock();
    if info.madt_address != 0 {
        Some(info.madt_address)
    } else {
        None
    }
}

/// Retorna endereço da MCFG (para ECAM).
pub fn get_mcfg_address() -> Option<u64> {
    let info = ACPI_INFO.lock();
    if info.mcfg_address != 0 {
        Some(info.mcfg_address)
    } else {
        None
    }
}

/// Verifica se ACPI está disponível.
pub fn is_available() -> bool {
    ACPI_INFO.lock().initialized
}

/// Desliga subsistema ACPI.
pub fn shutdown() {
    crate::kinfo!("(ACPI) Shutdown");
}
