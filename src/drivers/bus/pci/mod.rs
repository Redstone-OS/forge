//! # PCI/PCIe Bus Driver
//!
//! Este módulo implementa o driver do barramento **PCI (Peripheral Component
//! Interconnect)** e sua extensão PCIe. É o barramento de expansão mais
//! importante em sistemas modernos.
//!
//! ## Dispositivos Típicos:
//! - GPUs (NVIDIA, AMD, Intel)
//! - NICs (Intel, Realtek)
//! - Storage Controllers (NVMe, AHCI)
//! - USB Host Controllers (xHCI, EHCI)
//! - Audio (Intel HDA)
//!
//! ## Endereçamento:
//! Cada dispositivo PCI tem um endereço único:
//! - **Segment**: Domínio PCIe (geralmente 0)
//! - **Bus**: Barramento (0-255)
//! - **Device**: Slot no barramento (0-31)
//! - **Function**: Função dentro do slot (0-7)
//!
//! ## Registradores de Configuração:
//! Cada dispositivo tem 256 bytes (PCI) ou 4KB (PCIe) de espaço de config.
//! Acessamos via portas 0xCF8/0xCFC ou MMIO (ECAM).
//!
//! ## Fluxo:
//! 1. init() - Detecta ECAM ou usa portas legadas
//! 2. scan() - Enumera todos os barramentos
//! 3. Para cada dispositivo: cria Device e registra na base

pub mod access; // Acesso a registradores de config
pub mod config; // Layout do espaço de configuração
pub mod device; // Estrutura PciDevice

// Re-exports
pub use access::{read_config, write_config};
pub use device::PciDevice;

use crate::drivers::base::bus::{Bus, BusAddress, BusType};
use crate::drivers::base::device::{Device, DeviceId};
use crate::drivers::base::driver::DeviceType;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Porta de endereço PCI (CONFIG_ADDRESS).
pub const PCI_CONFIG_ADDRESS: u16 = 0x0CF8;

/// Porta de dados PCI (CONFIG_DATA).
pub const PCI_CONFIG_DATA: u16 = 0x0CFC;

/// Número máximo de barramentos PCI.
pub const MAX_PCI_BUSES: u8 = 255;

/// Número máximo de dispositivos por barramento.
pub const MAX_PCI_DEVICES: u8 = 32;

/// Número máximo de funções por dispositivo.
pub const MAX_PCI_FUNCTIONS: u8 = 8;

// =============================================================================
// ENDEREÇO PCI
// =============================================================================

/// Endereço completo de um dispositivo PCI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciAddress {
    /// Segmento PCIe (domínio). Geralmente 0.
    pub segment: u16,
    /// Número do barramento (0-255).
    pub bus: u8,
    /// Número do slot/dispositivo (0-31).
    pub device: u8,
    /// Número da função (0-7).
    pub function: u8,
}

impl PciAddress {
    /// Cria novo endereço PCI.
    pub const fn new(segment: u16, bus: u8, device: u8, function: u8) -> Self {
        Self {
            segment,
            bus,
            device,
            function,
        }
    }

    /// Formata o endereço no estilo padrão: SSSS:BB:DD.F
    pub fn format(&self) -> [u8; 16] {
        // Retorna buffer com formato "0000:00:00.0"
        let mut buf = [0u8; 16];
        // Simplificação: apenas retorna zeros
        buf
    }

    /// Converte para BusAddress da base.
    pub fn to_bus_address(&self) -> BusAddress {
        BusAddress::Pci {
            segment: self.segment,
            bus: self.bus,
            device: self.device,
            function: self.function,
        }
    }
}

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Lista de dispositivos PCI descobertos.
static PCI_DEVICES: Spinlock<Vec<PciDevice>> = Spinlock::new(Vec::new());

/// Flag indicando se ECAM (PCIe) está disponível.
static ECAM_AVAILABLE: Spinlock<bool> = Spinlock::new(false);

/// Base MMIO do ECAM (se disponível).
static ECAM_BASE: Spinlock<u64> = Spinlock::new(0);

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// IMPLEMENTAÇÃO DO BUS TRAIT
// =============================================================================

/// Implementação do barramento PCI.
pub struct PciBus;

impl Bus for PciBus {
    fn name(&self) -> &'static str {
        "PCI Express Root Complex"
    }

    fn bus_type(&self) -> BusType {
        BusType::Pci
    }

    fn scan(&self) -> Vec<Device> {
        scan()
    }

    fn reset_device(&self, dev: &mut Device) -> bool {
        // PCI reset via FLR (Function Level Reset) ou secundário
        crate::kwarn!("(PCI) reset_device() usando reset simples");

        if let BusAddress::Pci {
            bus,
            device,
            function,
            ..
        } = dev.bus_address
        {
            let addr = PciAddress::new(0, bus, device, function);

            // Escreve bit de reset no Command Register
            let cmd = read_config(addr, config::PCI_COMMAND);
            write_config(addr, config::PCI_COMMAND, 0); // Desabilita
                                                        // Pequeno delay aqui seria ideal
            write_config(addr, config::PCI_COMMAND, cmd); // Re-habilita

            true
        } else {
            false
        }
    }

    fn shutdown(&self) {
        crate::kinfo!("(PCI) Desligando barramento PCI...");
        // Desabilita bus mastering em todos os dispositivos
        let devices = PCI_DEVICES.lock();
        for dev in devices.iter() {
            disable_bus_master(dev.address);
        }
    }
}

/// Instância global do barramento PCI.
static PCI_BUS: PciBus = PciBus;

/// Retorna referência ao barramento PCI.
pub fn get_bus() -> Arc<dyn Bus> {
    Arc::new(PciBus)
}

// =============================================================================
// FUNÇÕES DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa o subsistema PCI.
pub fn init() {
    crate::kinfo!("(PCI) Inicializando PCI/PCIe...");

    // Tenta detectar ECAM via ACPI (MCFG table)
    if let Some(ecam_base) = detect_ecam() {
        *ECAM_BASE.lock() = ecam_base;
        *ECAM_AVAILABLE.lock() = true;
        crate::kinfo!("(PCI) ECAM detectado em:", ecam_base);
    } else {
        crate::kinfo!("(PCI) Usando acesso via portas legadas (0xCF8/0xCFC)");
    }

    *INITIALIZED.lock() = true;

    // Registra na base
    crate::drivers::base::bus::register(Arc::new(PciBus));

    crate::kinfo!("(PCI) Subsistema inicializado");
}

/// Escaneia todos os barramentos PCI.
pub fn scan() -> Vec<Device> {
    crate::kinfo!("(PCI) Escaneando barramentos PCI...");

    let mut devices = Vec::new();
    let mut pci_devices = PCI_DEVICES.lock();
    pci_devices.clear();

    // Escaneia barramento 0
    scan_bus(0, &mut *pci_devices);

    crate::kinfo!("(PCI) Encontrados", pci_devices.len(), "dispositivos PCI");

    // Converte para Device da base
    for pci_dev in pci_devices.iter() {
        let dev = pci_device_to_base_device(pci_dev);
        devices.push(dev);
    }

    devices
}

/// Escaneia um barramento específico.
fn scan_bus(bus: u8, devices: &mut Vec<PciDevice>) {
    for device in 0..MAX_PCI_DEVICES {
        scan_device(bus, device, devices);
    }
}

/// Escaneia um dispositivo (todas as funções).
fn scan_device(bus: u8, device: u8, devices: &mut Vec<PciDevice>) {
    let addr = PciAddress::new(0, bus, device, 0);

    // Lê Vendor ID para verificar se dispositivo existe
    let vendor_id = read_config(addr, config::PCI_VENDOR_ID) as u16;

    // 0xFFFF significa slot vazio
    if vendor_id == 0xFFFF {
        return;
    }

    // Escaneia função 0
    scan_function(bus, device, 0, devices);

    // Verifica se é multi-função
    let header_type = read_config(addr, config::PCI_HEADER_TYPE) as u8;
    if header_type & 0x80 != 0 {
        // Multi-função: escaneia funções 1-7
        for function in 1..MAX_PCI_FUNCTIONS {
            scan_function(bus, device, function, devices);
        }
    }
}

/// Escaneia uma função específica.
fn scan_function(bus: u8, device: u8, function: u8, devices: &mut Vec<PciDevice>) {
    let addr = PciAddress::new(0, bus, device, function);

    let vendor_id = read_config(addr, config::PCI_VENDOR_ID) as u16;
    if vendor_id == 0xFFFF {
        return;
    }

    let device_id = read_config(addr, config::PCI_DEVICE_ID) as u16;
    let class = read_config(addr, config::PCI_CLASS) as u8;
    let subclass = read_config(addr, config::PCI_SUBCLASS) as u8;
    let prog_if = read_config(addr, config::PCI_PROG_IF) as u8;
    let revision = read_config(addr, config::PCI_REVISION) as u8;

    let pci_dev = PciDevice {
        address: addr,
        vendor_id,
        device_id,
        class_code: class,
        subclass_code: subclass,
        prog_if,
        revision,
        header_type: read_config(addr, config::PCI_HEADER_TYPE) as u8 & 0x7F,
        interrupt_line: read_config(addr, config::PCI_INTERRUPT_LINE) as u8,
        interrupt_pin: read_config(addr, config::PCI_INTERRUPT_PIN) as u8,
        bars: read_all_bars(addr),
    };

    crate::kinfo!(
        "(PCI) Encontrado:",
        vendor_id,
        ":",
        device_id,
        "classe",
        class,
        "/",
        subclass
    );

    // Verifica se é bridge PCI-to-PCI
    if class == 0x06 && subclass == 0x04 {
        // É uma bridge - escaneia barramento secundário
        let secondary_bus = read_config(addr, 0x19) as u8;
        crate::kinfo!("(PCI) Bridge para barramento", secondary_bus);
        scan_bus(secondary_bus, devices);
    }

    devices.push(pci_dev);
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Lê todos os BARs de um dispositivo.
fn read_all_bars(addr: PciAddress) -> [u32; 6] {
    [
        read_config(addr, config::PCI_BAR0),
        read_config(addr, config::PCI_BAR1),
        read_config(addr, config::PCI_BAR2),
        read_config(addr, config::PCI_BAR3),
        read_config(addr, config::PCI_BAR4),
        read_config(addr, config::PCI_BAR5),
    ]
}

/// Converte PciDevice para Device da base.
fn pci_device_to_base_device(pci_dev: &PciDevice) -> Device {
    let dev_type = class_to_device_type(pci_dev.class_code, pci_dev.subclass_code);
    let name = get_device_name(pci_dev.vendor_id, pci_dev.device_id);

    let mut dev = Device::new(
        name,
        BusType::Pci,
        pci_dev.address.to_bus_address(),
        dev_type,
    );

    dev.set_ids(
        pci_dev.vendor_id,
        pci_dev.device_id,
        pci_dev.class_code,
        pci_dev.subclass_code,
        pci_dev.revision,
    );

    dev
}

/// Mapeia classe PCI para DeviceType.
fn class_to_device_type(class: u8, subclass: u8) -> DeviceType {
    match class {
        0x01 => DeviceType::Storage, // Mass Storage
        0x02 => DeviceType::Network, // Network Controller
        0x03 => DeviceType::Display, // Display Controller
        0x04 => DeviceType::Audio,   // Multimedia (Audio)
        0x06 => DeviceType::Bus,     // Bridge
        0x0C => match subclass {
            0x03 => DeviceType::Bus, // USB Controller
            _ => DeviceType::Bus,
        },
        _ => DeviceType::Generic,
    }
}

/// Retorna nome legível para um dispositivo.
fn get_device_name(vendor: u16, device: u16) -> &'static str {
    // TODO: Tabela de nomes de dispositivos
    match vendor {
        0x8086 => "Intel Device",
        0x10DE => "NVIDIA Device",
        0x1002 => "AMD Device",
        0x1AF4 => "VirtIO Device",
        _ => "PCI Device",
    }
}

/// Detecta ECAM via ACPI MCFG.
fn detect_ecam() -> Option<u64> {
    // TODO: Ler tabela MCFG do ACPI
    crate::kwarn!("(PCI) detect_ecam() - MCFG não implementado");
    None
}

/// Desabilita bus mastering em um dispositivo.
fn disable_bus_master(addr: PciAddress) {
    let cmd = read_config(addr, config::PCI_COMMAND);
    write_config(addr, config::PCI_COMMAND, cmd & !0x04);
}

/// Habilita bus mastering em um dispositivo.
pub fn enable_bus_master(addr: PciAddress) {
    let cmd = read_config(addr, config::PCI_COMMAND);
    write_config(addr, config::PCI_COMMAND, cmd | 0x07); // I/O + Memory + Bus Master
}

/// Retorna número de dispositivos PCI.
pub fn device_count() -> usize {
    PCI_DEVICES.lock().len()
}

/// Busca dispositivo PCI por classe/subclasse.
pub fn find_by_class(class: u8, subclass: u8) -> Vec<PciDevice> {
    PCI_DEVICES
        .lock()
        .iter()
        .filter(|d| d.class_code == class && d.subclass_code == subclass)
        .cloned()
        .collect()
}

/// Busca dispositivo PCI por vendor/device ID.
pub fn find_by_id(vendor: u16, device: u16) -> Option<PciDevice> {
    PCI_DEVICES
        .lock()
        .iter()
        .find(|d| d.vendor_id == vendor && d.device_id == device)
        .cloned()
}

/// Desliga o barramento PCI.
pub fn shutdown() {
    PCI_BUS.shutdown();
}
