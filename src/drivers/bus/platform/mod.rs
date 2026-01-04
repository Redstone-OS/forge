//! # Platform Bus Driver
//!
//! Este módulo implementa o **Platform Bus** - um barramento virtual para
//! dispositivos cujos recursos são conhecidos a priori (via firmware ou
//! configuração fixa).
//!
//! ## Dispositivos Típicos:
//! - Controladores de interrupção (PIC, APIC, IOAPIC)
//! - Timers (PIT, HPET, LAPIC Timer)
//! - RTC (Real Time Clock)
//! - Serial ports (COM1-COM4)
//! - PS/2 Controller
//! - Speaker
//!
//! ## Diferença para outros buses:
//! - **PCI**: Dispositivos são descobertos dinamicamente via scan
//! - **Platform**: Dispositivos são conhecidos estaticamente
//!
//! ## Uso:
//! Drivers de platform device sabem exatamente quais recursos precisam
//! e os solicitam diretamente via `base::resource::request_*()`.

use crate::drivers::base::bus::{Bus, BusAddress, BusType};
use crate::drivers::base::device::Device;
use crate::drivers::base::driver::DeviceType;
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Lista de dispositivos platform registrados.
static PLATFORM_DEVICES: Spinlock<Vec<PlatformDevice>> = Spinlock::new(Vec::new());

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// ESTRUTURA DE DISPOSITIVO PLATFORM
// =============================================================================

/// Representa um dispositivo platform conhecido.
#[derive(Debug, Clone)]
pub struct PlatformDevice {
    /// Nome do dispositivo.
    pub name: &'static str,

    /// Tipo funcional.
    pub device_type: DeviceType,

    /// Endereço I/O base (se aplicável).
    pub io_base: Option<u16>,

    /// Tamanho da região I/O.
    pub io_size: u16,

    /// Endereço MMIO base (se aplicável).
    pub mmio_base: Option<u64>,

    /// Tamanho da região MMIO.
    pub mmio_size: u64,

    /// IRQ (se aplicável).
    pub irq: Option<u8>,

    /// Ordem de prioridade (menor = inicializa primeiro).
    pub priority: u8,
}

// =============================================================================
// IMPLEMENTAÇÃO DO BUS TRAIT
// =============================================================================

/// Implementação do barramento Platform.
pub struct PlatformBus;

impl Bus for PlatformBus {
    fn name(&self) -> &'static str {
        "Platform Bus"
    }

    fn bus_type(&self) -> BusType {
        BusType::Platform
    }

    fn scan(&self) -> Vec<Device> {
        scan()
    }

    fn reset_device(&self, _dev: &mut Device) -> bool {
        // Platform devices geralmente não suportam reset via bus
        crate::kwarn!("(Platform) Reset não suportado para platform devices");
        false
    }

    fn shutdown(&self) {
        crate::kinfo!("(Platform) Desligando platform bus...");
    }
}

/// Retorna referência ao barramento Platform.
pub fn get_bus() -> Arc<dyn Bus> {
    Arc::new(PlatformBus)
}

// =============================================================================
// FUNÇÕES DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa o Platform Bus e registra dispositivos conhecidos.
pub fn init() {
    crate::kinfo!("(Platform) Inicializando Platform Bus...");

    let mut devices = PLATFORM_DEVICES.lock();

    // Registra dispositivos conhecidos estaticamente
    register_known_devices(&mut devices);

    *INITIALIZED.lock() = true;

    // Registra na base
    crate::drivers::base::bus::register(Arc::new(PlatformBus));

    crate::kinfo!("(Platform) Registrados", devices.len(), "dispositivos");
}

/// Registra dispositivos platform conhecidos.
fn register_known_devices(devices: &mut Vec<PlatformDevice>) {
    // PIT (Programmable Interval Timer)
    devices.push(PlatformDevice {
        name: "PIT Timer",
        device_type: DeviceType::Timer,
        io_base: Some(0x40),
        io_size: 4,
        mmio_base: None,
        mmio_size: 0,
        irq: Some(0),
        priority: 1, // Crítico - inicializa cedo
    });

    // RTC (Real Time Clock)
    devices.push(PlatformDevice {
        name: "CMOS RTC",
        device_type: DeviceType::Timer,
        io_base: Some(0x70),
        io_size: 2,
        mmio_base: None,
        mmio_size: 0,
        irq: Some(8),
        priority: 2,
    });

    // PS/2 Controller
    devices.push(PlatformDevice {
        name: "PS/2 Controller",
        device_type: DeviceType::Input,
        io_base: Some(0x60),
        io_size: 8, // 0x60 e 0x64
        mmio_base: None,
        mmio_size: 0,
        irq: Some(1), // IRQ 1 para teclado, IRQ 12 para mouse
        priority: 10,
    });

    // COM1 (Serial Port)
    devices.push(PlatformDevice {
        name: "COM1 Serial",
        device_type: DeviceType::Serial,
        io_base: Some(0x3F8),
        io_size: 8,
        mmio_base: None,
        mmio_size: 0,
        irq: Some(4),
        priority: 5,
    });

    // COM2 (Serial Port)
    devices.push(PlatformDevice {
        name: "COM2 Serial",
        device_type: DeviceType::Serial,
        io_base: Some(0x2F8),
        io_size: 8,
        mmio_base: None,
        mmio_size: 0,
        irq: Some(3),
        priority: 6,
    });

    // Speaker
    devices.push(PlatformDevice {
        name: "PC Speaker",
        device_type: DeviceType::Audio,
        io_base: Some(0x61),
        io_size: 1,
        mmio_base: None,
        mmio_size: 0,
        irq: None,
        priority: 100, // Baixa prioridade
    });

    // PIC (Programmable Interrupt Controller)
    devices.push(PlatformDevice {
        name: "8259 PIC",
        device_type: DeviceType::Controller,
        io_base: Some(0x20),
        io_size: 2, // Master: 0x20-0x21, Slave: 0xA0-0xA1
        mmio_base: None,
        mmio_size: 0,
        irq: None,
        priority: 0, // Mais crítico
    });

    // DMA Controller
    devices.push(PlatformDevice {
        name: "8237 DMA",
        device_type: DeviceType::Controller,
        io_base: Some(0x00),
        io_size: 16,
        mmio_base: None,
        mmio_size: 0,
        irq: None,
        priority: 3,
    });
}

/// Escaneia dispositivos platform.
pub fn scan() -> Vec<Device> {
    crate::kinfo!("(Platform) Listando dispositivos platform...");

    let pdevs = PLATFORM_DEVICES.lock();
    let mut devices = Vec::new();

    // Ordena por prioridade
    let mut sorted: Vec<_> = pdevs.iter().collect();
    sorted.sort_by_key(|d| d.priority);

    for pdev in sorted {
        let bus_addr = if let Some(io) = pdev.io_base {
            BusAddress::IoPort {
                port: io,
                count: pdev.io_size,
            }
        } else if let Some(mmio) = pdev.mmio_base {
            BusAddress::Mmio {
                base: mmio,
                size: pdev.mmio_size,
            }
        } else {
            BusAddress::None
        };

        let mut dev = Device::new(pdev.name, BusType::Platform, bus_addr, pdev.device_type);

        // Platform devices não tem vendor/device ID
        dev.set_ids(0, 0, 0, 0, 0);

        devices.push(dev);
    }

    devices
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Registra um novo dispositivo platform em runtime.
pub fn register_device(device: PlatformDevice) {
    crate::kinfo!("(Platform) Registrando:", device.name);
    PLATFORM_DEVICES.lock().push(device);
}

/// Retorna número de dispositivos platform.
pub fn device_count() -> usize {
    PLATFORM_DEVICES.lock().len()
}

/// Desliga o platform bus.
pub fn shutdown() {
    crate::kinfo!("(Platform) Shutdown");
}
