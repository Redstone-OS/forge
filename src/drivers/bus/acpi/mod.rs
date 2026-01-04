//! # Subsistema ACPI (Advanced Configuration and Power Interface)
//!
//! O ACPI é o barramento lógico primário para descoberta de hardware estático
//! (CPUs, IOAPICs, Timers) e gerenciamento de energia.
//!
//! Este módulo implementa a descoberta das tabelas ACPI e fornece a base para
//! o suporte a multi-processamento e desligamento do sistema.

pub mod apic;
pub mod fadt;
pub mod madt;
pub mod tables;

use self::tables::{AcpiTables, SdtHeader};
use super::super::base::bus::{Bus, BusType};
use super::super::base::device::{Device, DeviceId, DeviceState};
use super::super::base::driver::DeviceType;
use crate::sync::Spinlock;
use alloc::vec::Vec;

/// Implementação do Barramento ACPI para o RDM
pub struct AcpiBus {
    tables: Spinlock<AcpiTables>,
}

impl AcpiBus {
    pub const fn new() -> Self {
        Self {
            tables: Spinlock::new(AcpiTables::new()),
        }
    }

    /// Inicializa a busca por tabelas ACPI.
    /// Deve ser chamado com o endereço do RSDP fornecido pelo bootloader.
    pub fn init_with_rsdp(&self, rsdp_addr: u64) {
        let mut tables = self.tables.lock();
        tables.init(rsdp_addr);

        // Exemplo: Buscar e logar tabelas críticas
        if let Some(madt_ptr) = tables.find_table(b"APIC") {
            let madt = unsafe { &*(madt_ptr.as_ptr() as *const madt::MadtHeader) };
            let parser = madt::MadtParser::new(madt);
            parser.parse();
        }

        if let Some(fadt_ptr) = tables.find_table(b"FACP") {
            let fadt = unsafe { &*(fadt_ptr.as_ptr() as *const fadt::FadtHeader) };
            crate::kdebug!("(ACPI) FADT Detectada. Suporte a 8042:", fadt.has_8042());
        }
    }
}

impl Bus for AcpiBus {
    fn name(&self) -> &'static str {
        "ACPI Logical Bus"
    }

    fn bus_type(&self) -> BusType {
        BusType::Acpi
    }

    /// O barramento ACPI "escaneia" o hardware através das tabelas SDT.
    /// Ele registra CPUs e controladores de sistema como dispositivos.
    fn scan(&self) -> Vec<Device> {
        let mut devices = Vec::new();

        // 1. Registrar a própria ACPI como um dispositivo de sistema
        let acpi_dev = Device::new(
            DeviceId(0xAC91), // ID arbitrário para ACPI Central
            "acpi-root",
            BusType::System,
            super::super::base::bus::BusAddress::None,
            DeviceType::Controller,
        );
        devices.push(acpi_dev);

        // TODO: Iterar sobre todas as tabelas encontradas e registrar como dispositivos individuais
        // Isso permitirá que o RDM associe drivers de gerenciamento de energia a tabelas específicas.

        devices
    }

    fn reset_device(&self, _dev: &mut Device) -> bool {
        // O reset geral do sistema via ACPI é feito no FADT, não em dispositivos individuais.
        false
    }
}

// Singleton para o barramento ACPI
static ACPI_BUS_INSTANCE: AcpiBus = AcpiBus::new();

/// Inicializa o subsistema ACPI e registra no RDM
pub fn init(rsdp_addr: u64) {
    crate::kinfo!("(ACPI) Inicializando barramento lógico...");
    ACPI_BUS_INSTANCE.init_with_rsdp(rsdp_addr);

    // Registra-se como um barramento disponível no RDM
    // crate::drivers::base::bus::register(Arc::new(ACPI_BUS_INSTANCE));
}
