//! /dev/rtc - Real-Time Clock (Relógio de Tempo Real)
//!
//! Dispositivo essencial do kernel para timestamps e scheduler.

use crate::fs::devfs::char_device::CharDevice;
use crate::fs::devfs::device::{Device, DeviceNumber, DeviceType};

/// /dev/rtc device
pub struct RtcDevice {
    dev: DeviceNumber,
}

impl RtcDevice {
    /// Cria um novo /dev/rtc
    pub const fn new() -> Self {
        Self {
            dev: DeviceNumber::new(10, 135),
        }
    }
}

impl Device for RtcDevice {
    fn name(&self) -> &str {
        "rtc"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Character
    }

    fn device_number(&self) -> DeviceNumber {
        self.dev
    }

    fn read(&self, _buf: &mut [u8]) -> Result<usize, &'static str> {
        // TODO: Implementar leitura do RTC
        // - Ler registradores CMOS (0x70/0x71)
        // - Retornar timestamp Unix
        Err("rtc read not implemented")
    }

    fn write(&self, _buf: &[u8]) -> Result<usize, &'static str> {
        // TODO: Implementar escrita no RTC
        // - Escrever registradores CMOS
        // - Atualizar relógio
        Err("rtc write not implemented")
    }

    fn ioctl(&self, cmd: usize, _arg: usize) -> Result<usize, &'static str> {
        match cmd {
            // TODO: Implementar ioctls do RTC
            // - RTC_RD_TIME: Ler tempo
            // - RTC_SET_TIME: Configurar tempo
            // - RTC_ALM_SET: Configurar alarme
            _ => Err("Unknown ioctl"),
        }
    }
}

impl CharDevice for RtcDevice {}

impl Default for RtcDevice {
    fn default() -> Self {
        Self::new()
    }
}
