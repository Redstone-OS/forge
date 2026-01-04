//! # Camada de Barramentos do RDS (Bus Layer)
//!
//! Este módulo é a **rodoviária central** do sistema de hardware. Aqui ficam
//! os drivers de barramento que descobrem e conectam dispositivos ao kernel.
//!
//! ## Responsabilidades:
//! - **Descoberta**: Enumerar todos os dispositivos conectados
//! - **Identificação**: Ler IDs de vendor/device de cada hardware
//! - **Transporte**: Fornecer primitivas de comunicação (MMIO, I/O, DMA)
//! - **Hierarquia**: Manter árvore de dispositivos (ex: USB hub → dispositivos)
//!
//! ## Barramentos Suportados:
//!
//! ### Críticos (Fase 1):
//! - **PCI/PCIe**: Principal barramento de expansão moderno
//! - **VirtIO**: Paravirtualização para QEMU/KVM
//! - **Platform/ISA**: Dispositivos legados fixos
//!
//! ### Secundários (Fases Posteriores):
//! - **USB**: xHCI (USB 3.0), EHCI (USB 2.0)
//! - **ACPI**: Descoberta de dispositivos via tabelas BIOS/UEFI
//!
//! ## Integração com RDS Base:
//! Cada barramento implementa a trait `Bus` de `drivers::base::bus`.
//! Dispositivos descobertos são registrados via `base::register_device()`.

// =============================================================================
// SUBMÓDULOS DE BARRAMENTO
// =============================================================================

/// PCI/PCIe Bus - Barramento principal de expansão.
/// Usado por GPUs, NICs, NVMe, AHCI, etc.
pub mod pci;

/// VirtIO Bus - Paravirtualização de alta performance.
/// Usado por VirtIO-Blk, VirtIO-Net, VirtIO-GPU.
pub mod virtio;

/// Platform/ISA Bus - Dispositivos legados.
/// PS/2, PIT, RTC, Serial, etc.
pub mod platform;

/// ISA Bus Legacy - Compatibilidade com hardware antigo.
pub mod isa;

/// ACPI - Descoberta de hardware via tabelas BIOS/UEFI.
pub mod acpi;

/// USB Stack - Universal Serial Bus.
/// xHCI (USB 3.0), EHCI (USB 2.0), dispositivos USB.
pub mod usb;

// =============================================================================
// RE-EXPORTS PÚBLICOS
// =============================================================================

// Re-exporta tipos importantes para uso direto
pub use pci::{PciAddress, PciBus, PciDevice};
pub use platform::PlatformBus;
pub use virtio::{VirtioBus, VirtioDevice};

// =============================================================================
// IMPORTS
// =============================================================================

use crate::drivers::base::bus::{Bus, BusType};
use crate::drivers::base::device::Device;
// TODO: Revisar no futuro
#[allow(unused_imports)]
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// FUNÇÕES DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa todos os barramentos do sistema.
///
/// ## Ordem de Inicialização:
/// 1. Platform/ISA (dispositivos fixos conhecidos)
/// 2. ACPI (descoberta de tabelas)
/// 3. PCI (enumera barramento)
/// 4. VirtIO (dispositivos virtualizados)
/// 5. USB (host controllers, depois dispositivos)
///
/// Esta função é chamada durante o boot, após `base::init()`.
pub fn init() {
    crate::kinfo!("(Bus Layer) Inicializando barramentos...");

    // 1. Platform/ISA - Dispositivos fixos
    platform::init();
    isa::init();

    // 2. ACPI - Tabelas de hardware
    acpi::init();

    // 3. PCI - Barramento principal
    pci::init();

    // 4. VirtIO - Paravirtualização
    virtio::init();

    // 5. USB - Após PCI (controladores USB são PCI devices)
    usb::init();

    crate::kinfo!("(Bus Layer) Todos os barramentos inicializados!");
}

/// Escaneia todos os barramentos e registra dispositivos encontrados.
///
/// Chamado após init() para popular o sistema com hardware.
pub fn scan_all() -> Vec<Device> {
    crate::kinfo!("(Bus Layer) Iniciando scan global de hardware...");

    let mut all_devices = Vec::new();

    // Platform (dispositivos fixos)
    all_devices.extend(platform::scan());

    // PCI (barramento de expansão)
    all_devices.extend(pci::scan());

    // VirtIO (paravirtualização)
    all_devices.extend(virtio::scan());

    // USB (depende de xHCI estar funcionando)
    // all_devices.extend(usb::scan()); // TODO: Habilitar após xHCI

    crate::kinfo!(
        "(Bus Layer) Scan completo. Total:",
        all_devices.len(),
        "dispositivos"
    );

    all_devices
}

/// Desliga todos os barramentos de forma ordenada.
///
/// Ordem reversa: USB → VirtIO → PCI → ACPI → Platform
pub fn shutdown() {
    crate::kinfo!("(Bus Layer) Iniciando shutdown dos barramentos...");

    usb::shutdown();
    virtio::shutdown();
    pci::shutdown();
    acpi::shutdown();
    platform::shutdown();

    crate::kinfo!("(Bus Layer) Shutdown completo!");
}

// =============================================================================
// FUNÇÕES DE CONSULTA
// =============================================================================

/// Busca um barramento específico pelo tipo.
pub fn get_bus(bus_type: BusType) -> Option<Arc<dyn Bus>> {
    match bus_type {
        BusType::Pci => Some(pci::get_bus()),
        BusType::Virtio => Some(virtio::get_bus()),
        BusType::Platform => Some(platform::get_bus()),
        BusType::Isa => Some(isa::get_bus()),
        _ => None,
    }
}

/// Retorna estatísticas dos barramentos.
pub fn get_stats() -> BusStats {
    BusStats {
        pci_devices: pci::device_count(),
        virtio_devices: virtio::device_count(),
        usb_devices: usb::device_count(),
        platform_devices: platform::device_count(),
    }
}

/// Estatísticas dos barramentos.
#[derive(Debug, Clone)]
pub struct BusStats {
    pub pci_devices: usize,
    pub virtio_devices: usize,
    pub usb_devices: usize,
    pub platform_devices: usize,
}
