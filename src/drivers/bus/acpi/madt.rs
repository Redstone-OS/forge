//! # MADT (Multiple APIC Description Table)
//!
//! Descreve os controladores de interrupção (APICs) e a topologia de CPUs.
//! Essencial para habilitar o processamento multi-core (SMP).

use super::tables::SdtHeader;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MadtHeader {
    pub header: SdtHeader,
    pub lapic_address: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum MadtEntryType {
    LocalApic = 0,
    IoApic = 1,
    InterruptOverride = 2,
    NmiSource = 3,
    LocalApicAddressOverride = 5,
    ProcessorLocalX2Apic = 9,
}

/// Representa uma CPU física ou lógica encontrada na MADT
pub struct CpuInfo {
    pub id: u8,
    pub apic_id: u8,
    pub enabled: bool,
}

/// Representa um controlador de IOAPIC encontrado na MADT
pub struct IoApicInfo {
    pub id: u8,
    pub address: u32,
    pub gsib: u32, // Global System Interrupt Base
}

pub struct MadtParser<'a> {
    header: &'a MadtHeader,
}

impl<'a> MadtParser<'a> {
    pub fn new(header: &'a MadtHeader) -> Self {
        Self { header }
    }

    /// Itera sobre as entradas da MADT para descobrir CPUs e IOAPICs
    pub fn parse(&self) {
        let table_len = self.header.header.length;
        let mut offset = core::mem::size_of::<MadtHeader>();
        let ptr = self.header as *const _ as *const u8;

        crate::ktrace!(
            "(ACPI) Parsing MADT, Local APIC Addr:",
            self.header.lapic_address as u64
        );

        while offset < table_len as usize {
            let entry_type = unsafe { *ptr.add(offset) };
            let entry_len = unsafe { *ptr.add(offset + 1) };

            match entry_type {
                0 => {
                    // Local APIC
                    let apic_id = unsafe { *ptr.add(offset + 3) };
                    let flags = unsafe { *(ptr.add(offset + 4) as *const u32) };
                    if flags & 1 != 0 {
                        crate::kinfo!("(ACPI) CPU Detectada - APIC ID:", apic_id as u64);
                    }
                }
                1 => {
                    // I/O APIC
                    let id = unsafe { *ptr.add(offset + 2) };
                    let addr = unsafe { *(ptr.add(offset + 4) as *const u32) };
                    let gsib = unsafe { *(ptr.add(offset + 8) as *const u32) };
                    crate::kinfo!("(ACPI) IOAPIC Detectado - ID:", id as u64);
                    crate::kdebug!("  -> Endereço:", addr as u64);
                    crate::kdebug!("  -> GSIB Base:", gsib as u64);
                }
                2 => {
                    // Interrupt Source Override
                    let bus = unsafe { *ptr.add(offset + 2) };
                    let source = unsafe { *ptr.add(offset + 3) };
                    let gsi = unsafe { *(ptr.add(offset + 4) as *const u32) };
                    crate::kdebug!("(ACPI) IRQ Override: Bus", bus as u64);
                    crate::ktrace!("  -> Source IRQ:", source as u64);
                    crate::ktrace!("  -> Target GSI:", gsi as u64);
                }
                _ => {}
            }

            offset += entry_len as usize;
        }
    }
}
