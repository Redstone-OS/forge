//! # Platform Bus
//!
//! Gerencia dispositivos que estão diretamente mapeados na memória ou I/O e
//! não podem ser descobertos automaticamente por barramentos como PCI ou USB.
//! É comumente usado para dispositivos fixos da placa-mãe ou em sistemas SoC.

use super::super::base::bus::{Bus, BusAddress, BusType};
use super::super::base::device::{Device, DeviceId, DeviceState};
use super::super::base::driver::DeviceType;
use crate::sync::Spinlock;
use alloc::vec::Vec;

pub struct PlatformBus {
    /// Lista de dispositivos registrados manualmente neste barramento
    static_devices: Spinlock<Vec<Device>>,
}

impl PlatformBus {
    pub const fn new() -> Self {
        Self {
            static_devices: Spinlock::new(Vec::new()),
        }
    }

    /// Adiciona um novo dispositivo de plataforma ao sistema.
    /// Como este barramento não tem auto-discovery, o kernel deve registrar
    /// os dispositivos conhecidos durante o boot.
    pub fn add_device(&self, dev: Device) {
        self.static_devices.lock().push(dev);
    }
}

impl Bus for PlatformBus {
    fn name(&self) -> &'static str {
        "System Platform Bus"
    }

    fn bus_type(&self) -> BusType {
        BusType::Platform
    }

    /// Retorna a lista de dispositivos que foram registrados manualmente
    fn scan(&self) -> Vec<Device> {
        let devices = self.static_devices.lock();
        crate::kdebug!(
            "(Platform) Escaneando dispositivos estáticos. Total:",
            devices.len() as u64
        );
        devices.clone()
    }

    fn reset_device(&self, _dev: &mut Device) -> bool {
        // Dispositivos de plataforma geralmente não possuem um mecanismo de reset genérico
        false
    }
}

static PLATFORM_BUS_INSTANCE: PlatformBus = PlatformBus::new();

/// Inicializa o subsistema de barramento de plataforma
pub fn init() {
    crate::kdebug!("(Platform) Inicializando barramento de plataforma...");

    // Futura integração com o RDM global
    // crate::drivers::base::bus::register(Arc::new(PLATFORM_BUS_INSTANCE));
}

/// Helper para registrar um dispositivo de plataforma de qualquer lugar do kernel
pub fn register_platform_device(dev: Device) {
    PLATFORM_BUS_INSTANCE.add_device(dev);
}
