//! # ACPI - Advanced Configuration and Power Interface
//!
//! Parser das tabelas ACPI para descoberta de hardware.
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                         ACPI                                │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │  BootInfo.rsdp_addr                                         │
//! │         │                                                   │
//! │         ▼                                                   │
//! │  ┌─────────────┐                                            │
//! │  │    RSDP     │  Root System Description Pointer           │
//! │  │  (v1 ou v2) │  Fornecido pelo bootloader                 │
//! │  └──────┬──────┘                                            │
//! │         │                                                   │
//! │         ▼                                                   │
//! │  ┌─────────────┐                                            │
//! │  │ RSDT / XSDT │  Tabela de ponteiros para outras tabelas   │
//! │  │ (32/64-bit) │                                            │
//! │  └──────┬──────┘                                            │
//! │         │                                                   │
//! │    ┌────┴────┬─────────┬─────────┐                          │
//! │    ▼         ▼         ▼         ▼                          │
//! │ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐                         │
//! │ │ MADT │ │ FADT │ │ DSDT │ │ ...  │                         │
//! │ │ CPUs │ │Power │ │ AML  │ │      │                         │
//! │ └──────┘ └──────┘ └──────┘ └──────┘                         │
//! │                                                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Fluxo de Inicialização
//!
//! 1. `init(rsdp_addr)` - Recebe endereço do bootloader
//! 2. Parseia RSDP → encontra RSDT/XSDT
//! 3. Itera sobre tabelas → encontra MADT
//! 4. Parseia MADT → descobre CPUs e I/O APICs
//! 5. Preenche `CpuTopology` global

pub mod madt;
pub mod tables;

use crate::rmm::virt::hhdm;
use tables::{Rsdp, SdtHeader};

/// Informações extraídas do ACPI
pub struct AcpiInfo {
    /// Endereço base dos Local APICs
    pub lapic_addr: u64,
    /// Número de CPUs detectadas
    pub cpu_count: usize,
    /// Número de I/O APICs detectadas
    pub ioapic_count: usize,
    /// Sistema tem PIC 8259 legado?
    pub has_legacy_pic: bool,
    /// CPUs detectadas (para inicializar topology)
    pub cpus: [Option<madt::CpuEntry>; madt::MAX_SUPPORTED_CPUS],
}

/// Inicializa o subsistema ACPI
///
/// # Safety
///
/// - `rsdp_addr` deve ser um endereço físico válido do RSDP
/// - HHDM deve estar inicializado para acessar tabelas via mapeamento direto
pub unsafe fn init(rsdp_addr: u64) -> Result<AcpiInfo, &'static str> {
    crate::kinfo!("(ACPI) Inicializando...");

    if rsdp_addr == 0 {
        return Err("RSDP address is null");
    }

    // 1. Parsear RSDP
    let rsdp_virt = hhdm::phys_to_virt(rsdp_addr);
    let rsdp = &*(rsdp_virt as *const Rsdp);

    if !rsdp.validate() {
        return Err("Invalid RSDP signature or checksum");
    }

    crate::kdebug!("(ACPI) RSDP válido, revisão:", rsdp.revision as u64);

    // 2. Encontrar RSDT ou XSDT (copiar valores de structs packed para evitar unaligned ref)
    let revision = rsdp.revision;
    let xsdt_addr = rsdp.xsdt_address;
    let rsdt_addr = rsdp.rsdt_address;

    let sdt_addr = if revision >= 2 && xsdt_addr != 0 {
        crate::kdebug!("(ACPI) Usando XSDT @", xsdt_addr);
        xsdt_addr
    } else {
        crate::kdebug!("(ACPI) Usando RSDT @", rsdt_addr as u64);
        rsdt_addr as u64
    };

    // 3. Encontrar MADT na tabela raiz
    let madt_addr = find_table(sdt_addr, rsdp.revision >= 2, b"APIC")?;
    crate::kdebug!("(ACPI) MADT encontrada @", madt_addr);

    // 4. Parsear MADT
    let madt_info = madt::parse(madt_addr)?;

    crate::kinfo!(
        "(ACPI) Detectados:",
        madt_info.cpu_count as u64,
        "CPUs,",
        madt_info.ioapic_count as u64,
        "I/O APICs"
    );

    Ok(AcpiInfo {
        lapic_addr: madt_info.lapic_addr,
        cpu_count: madt_info.cpu_count,
        ioapic_count: madt_info.ioapic_count,
        has_legacy_pic: madt_info.has_legacy_pic,
        cpus: madt_info.cpus,
    })
}

/// Encontra uma tabela ACPI pelo signature
unsafe fn find_table(
    sdt_addr: u64,
    is_xsdt: bool,
    signature: &[u8; 4],
) -> Result<u64, &'static str> {
    let sdt_virt = hhdm::phys_to_virt(sdt_addr);
    crate::kdebug!("(ACPI) SDT virt @", sdt_virt);
    let header = &*(sdt_virt as *const SdtHeader);
    crate::kdebug!("(ACPI) SDT length:", header.length as u64);

    // Calcular número de entradas
    let entry_size = if is_xsdt { 8 } else { 4 };
    let entries_start = sdt_virt + core::mem::size_of::<SdtHeader>() as u64;
    let entries_len = (header.length as usize - core::mem::size_of::<SdtHeader>()) / entry_size;
    crate::kdebug!("(ACPI) SDT entries:", entries_len as u64);
    crate::kdebug!("(ACPI) entries_start:", entries_start);

    for i in 0..entries_len {
        crate::kdebug!("(ACPI) Reading entry", i as u64);
        // SAFETY: Usar read_unaligned porque tabelas ACPI são packed e podem
        // ter ponteiros em endereços não alinhados a 8 bytes
        let entry_addr = if is_xsdt {
            core::ptr::read_unaligned((entries_start + (i * 8) as u64) as *const u64)
        } else {
            core::ptr::read_unaligned((entries_start + (i * 4) as u64) as *const u32) as u64
        };
        crate::kdebug!("(ACPI) Entry", i as u64, "@ phys", entry_addr);

        let entry_virt = hhdm::phys_to_virt(entry_addr);
        let entry_header = &*(entry_virt as *const SdtHeader);

        if &entry_header.signature == signature {
            return Ok(entry_addr);
        }
    }

    Err("Table not found")
}
