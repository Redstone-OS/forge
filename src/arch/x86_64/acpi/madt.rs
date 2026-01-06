//! # MADT - Multiple APIC Description Table
//!
//! Parser da tabela MADT para descoberta de CPUs e I/O APICs.
//!
//! ## Estrutura da MADT
//!
//! ```text
//! ┌──────────────────────────────────────────────┐
//! │              MADT Header (44 bytes)          │
//! │  - SDT Header (36 bytes)                     │
//! │  - Local APIC Address (4 bytes)              │
//! │  - Flags (4 bytes)                           │
//! ├──────────────────────────────────────────────┤
//! │              Entries (variável)              │
//! │  ┌────────────────────────────────────────┐  │
//! │  │ Type 0: Local APIC (CPU)               │  │
//! │  │   - ACPI Processor ID                  │  │
//! │  │   - APIC ID                            │  │
//! │  │   - Flags (enabled?)                   │  │
//! │  └────────────────────────────────────────┘  │
//! │  ┌────────────────────────────────────────┐  │
//! │  │ Type 1: I/O APIC                       │  │
//! │  │   - I/O APIC ID                        │  │
//! │  │   - Address                            │  │
//! │  │   - GSI Base                           │  │
//! │  └────────────────────────────────────────┘  │
//! │  ┌────────────────────────────────────────┐  │
//! │  │ Type 2: Interrupt Source Override      │  │
//! │  │   - IRQ remapping (ex: IRQ0 → GSI2)    │  │
//! │  └────────────────────────────────────────┘  │
//! │  ...                                         │
//! └──────────────────────────────────────────────┘
//! ```

use super::tables::SdtHeader;
use crate::rmm::virt::hhdm;

/// Número máximo de CPUs suportadas pelo parser
pub const MAX_SUPPORTED_CPUS: usize = 256;

/// Número máximo de I/O APICs
pub const MAX_IO_APICS: usize = 8;

/// Informações de uma CPU detectada
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuEntry {
    /// ACPI Processor ID
    pub acpi_id: u8,
    /// Local APIC ID (hardware ID)
    pub apic_id: u8,
    /// CPU está habilitada?
    pub enabled: bool,
    /// É o Bootstrap Processor?
    pub is_bsp: bool,
}

/// Informações de um I/O APIC
#[derive(Debug, Clone, Copy, Default)]
pub struct IoApicEntry {
    /// I/O APIC ID
    pub id: u8,
    /// Endereço físico do I/O APIC
    pub address: u32,
    /// Global System Interrupt base
    pub gsi_base: u32,
}

/// Resultado do parsing da MADT
pub struct MadtInfo {
    /// Endereço base dos Local APICs
    pub lapic_addr: u64,
    /// CPUs detectadas
    pub cpus: [Option<CpuEntry>; MAX_SUPPORTED_CPUS],
    /// Número de CPUs
    pub cpu_count: usize,
    /// I/O APICs detectados
    pub ioapics: [Option<IoApicEntry>; MAX_IO_APICS],
    /// Número de I/O APICs
    pub ioapic_count: usize,
    /// Sistema tem PIC 8259 legado?
    pub has_legacy_pic: bool,
}

impl MadtInfo {
    const fn new() -> Self {
        Self {
            lapic_addr: 0,
            cpus: [None; MAX_SUPPORTED_CPUS],
            cpu_count: 0,
            ioapics: [None; MAX_IO_APICS],
            ioapic_count: 0,
            has_legacy_pic: false,
        }
    }
}

// =============================================================================
// MADT Entry Types
// =============================================================================

/// Header genérico de entrada MADT
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct EntryHeader {
    entry_type: u8,
    length: u8,
}

/// Tipo 0: Processor Local APIC
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct LocalApicEntry {
    header: EntryHeader,
    acpi_processor_id: u8,
    apic_id: u8,
    flags: u32,
}

impl LocalApicEntry {
    fn is_enabled(&self) -> bool {
        (self.flags & 0x1) != 0 || (self.flags & 0x2) != 0
    }
}

/// Tipo 1: I/O APIC
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct IoApicEntryRaw {
    header: EntryHeader,
    io_apic_id: u8,
    reserved: u8,
    io_apic_address: u32,
    gsi_base: u32,
}

/// Tipo 9: Processor Local x2APIC (para sistemas com >255 CPUs)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct LocalX2ApicEntry {
    header: EntryHeader,
    reserved: u16,
    x2apic_id: u32,
    flags: u32,
    acpi_processor_uid: u32,
}

// =============================================================================
// Parser
// =============================================================================

/// Parseia a tabela MADT
///
/// # Safety
///
/// - `madt_phys_addr` deve ser um endereço físico válido
/// - HHDM deve estar inicializado
pub unsafe fn parse(madt_phys_addr: u64) -> Result<MadtInfo, &'static str> {
    let madt_virt = hhdm::phys_to_virt(madt_phys_addr);

    // Ler header SDT
    let header = &*(madt_virt as *const SdtHeader);

    // Campos específicos da MADT (após SDT header)
    let madt_data = madt_virt + core::mem::size_of::<SdtHeader>() as u64;
    let lapic_addr = *(madt_data as *const u32) as u64;
    let flags = *((madt_data + 4) as *const u32);

    let mut info = MadtInfo::new();
    info.lapic_addr = lapic_addr;
    info.has_legacy_pic = (flags & 0x1) != 0;

    // Detectar BSP pelo LAPIC ID atual
    let bsp_apic_id = crate::arch::x86_64::apic::lapic::id() as u8;

    // Iterar sobre entradas
    let entries_start = madt_data + 8;
    let entries_end = madt_virt + header.length as u64;
    let mut offset = entries_start;

    while offset < entries_end {
        let entry = &*(offset as *const EntryHeader);

        match entry.entry_type {
            // Tipo 0: Local APIC (CPU)
            0 => {
                let lapic = &*(offset as *const LocalApicEntry);
                if lapic.is_enabled() && info.cpu_count < MAX_SUPPORTED_CPUS {
                    info.cpus[info.cpu_count] = Some(CpuEntry {
                        acpi_id: lapic.acpi_processor_id,
                        apic_id: lapic.apic_id,
                        enabled: true,
                        is_bsp: lapic.apic_id == bsp_apic_id,
                    });
                    info.cpu_count += 1;
                }
            }

            // Tipo 1: I/O APIC
            1 => {
                let ioapic = &*(offset as *const IoApicEntryRaw);
                if info.ioapic_count < MAX_IO_APICS {
                    info.ioapics[info.ioapic_count] = Some(IoApicEntry {
                        id: ioapic.io_apic_id,
                        address: ioapic.io_apic_address,
                        gsi_base: ioapic.gsi_base,
                    });
                    info.ioapic_count += 1;
                }
            }

            // Tipo 9: Local x2APIC (CPUs com APIC ID > 255)
            9 => {
                let x2apic = &*(offset as *const LocalX2ApicEntry);
                let flags_enabled = (x2apic.flags & 0x1) != 0 || (x2apic.flags & 0x2) != 0;
                if flags_enabled && info.cpu_count < MAX_SUPPORTED_CPUS {
                    info.cpus[info.cpu_count] = Some(CpuEntry {
                        acpi_id: x2apic.acpi_processor_uid as u8,
                        apic_id: x2apic.x2apic_id as u8, // Truncado para compatibilidade
                        enabled: true,
                        is_bsp: x2apic.x2apic_id == bsp_apic_id as u32,
                    });
                    info.cpu_count += 1;
                }
            }

            // Outros tipos (ISO, NMI, etc.) - ignorar por enquanto
            _ => {}
        }

        // Avançar para próxima entrada
        if entry.length == 0 {
            break; // Evitar loop infinito
        }
        offset += entry.length as u64;
    }

    // Garantir que pelo menos o BSP foi detectado
    if info.cpu_count == 0 {
        info.cpus[0] = Some(CpuEntry {
            acpi_id: 0,
            apic_id: bsp_apic_id,
            enabled: true,
            is_bsp: true,
        });
        info.cpu_count = 1;
    }

    Ok(info)
}
