//! # ISA / LPC Bus (Industry Standard Architecture)
//!
//! O barramento legado para periféricos de baixa velocidade.
//! Em sistemas modernos, ele é implementado via LPC (Low Pin Count) ou
//! eSPI, mas emoldurado como ISA para compatibilidade.
//!
//! ## Dispositivos Comuns:
//! - **UART (16550)**: Portas Seriais (COM1 em 0x3F8).
//! - **8042 PS/2**: Teclado e Mouse.
//! - **Real Time Clock (RTC)**: CMOS RAM e Relógio.
//! - **PC Speaker**: Buzzer do sistema.
//! - **Floppy Controller**: Controlador de disquete (Opcional).

pub mod devices;

use super::super::base::bus::{Bus, BusAddress, BusType};
use super::super::base::device::{Device, DeviceId, DeviceState};
use super::super::base::driver::DeviceType;
use crate::sync::Spinlock;
use alloc::vec::Vec;

pub struct IsaBus;

impl IsaBus {
    pub const fn new() -> Self {
        Self
    }
}

impl Bus for IsaBus {
    fn name(&self) -> &'static str {
        "ISA/LPC Legacy Bus"
    }

    fn bus_type(&self) -> BusType {
        BusType::Isa
    }

    /// O barramento ISA não é "discoverable" (não há probing automático).
    /// Retornamos uma lista de dispositivos "bem conhecidos" (well-known)
    /// conforme a especificação padrão do PC.
    fn scan(&self) -> Vec<Device> {
        let mut devices = Vec::new();

        crate::kdebug!("(ISA) Instanciando dispositivos legados estáticos...");

        // 1. Serial COM1 (0x3F8)
        devices.push(Device::new(
            DeviceId(0x16550_1),
            "COM1",
            BusType::Isa,
            BusAddress::IoPort(0x3F8),
            DeviceType::Serial,
        ));

        // 2. PS/2 Controller (0x60, 0x64)
        devices.push(Device::new(
            DeviceId(0x8042),
            "PS2-Controller",
            BusType::Isa,
            BusAddress::IoPort(0x60),
            DeviceType::Controller,
        ));

        // 3. Real Time Clock (0x70)
        devices.push(Device::new(
            DeviceId(0x70),
            "RTC",
            BusType::Isa,
            BusAddress::IoPort(0x70),
            DeviceType::Timer,
        ));

        // 4. PC Speaker (0x61)
        devices.push(Device::new(
            DeviceId(0x61),
            "PC-Speaker",
            BusType::Isa,
            BusAddress::IoPort(0x61),
            DeviceType::Generic,
        ));

        devices
    }

    fn reset_device(&self, _dev: &mut Device) -> bool {
        // ISA não suporta reset individual de slot via software.
        false
    }
}

static ISA_BUS_INSTANCE: IsaBus = IsaBus.new();

/// Inicializa o subsistema ISA e registra os dispositivos estáticos no RDM
pub fn init() {
    crate::kinfo!("(ISA) Inicializando barramento legado...");

    // Como ISA é estático, o DriverManager chamará o scan e registrará os periféricos.
    // super::super::base::bus::register(Arc::new(ISA_BUS_INSTANCE));
}
