//! /dev/mem e /dev/kmem - Acesso direto à memória
//!
//! Dispositivos essenciais do kernel para debug.
//!
//! # Segurança
//! ⚠️ PERIGO: Acesso direto à memória física!
//! - Apenas root deve ter acesso
//! - Pode crashar o sistema se usado incorretamente
//! - Útil para debug e drivers

use crate::fs::devfs::char_device::CharDevice;
use crate::fs::devfs::device::{Device, DeviceNumber, DeviceType};

/// /dev/mem - Acesso à memória física
pub struct MemDevice {
    dev: DeviceNumber,
}

impl MemDevice {
    /// Cria um novo /dev/mem
    pub const fn new() -> Self {
        Self {
            dev: DeviceNumber::new(1, 1),
        }
    }
}

impl Device for MemDevice {
    fn name(&self) -> &str {
        "mem"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Character
    }

    fn device_number(&self) -> DeviceNumber {
        self.dev
    }

    fn read(&self, _buf: &mut [u8]) -> Result<usize, &'static str> {
        // TODO: Implementar leitura de memória física
        // - Verificar permissões (apenas root)
        // - Verificar endereço válido
        // - Copiar da memória física
        Err("mem read not implemented")
    }

    fn write(&self, _buf: &[u8]) -> Result<usize, &'static str> {
        // TODO: Implementar escrita em memória física
        // - Verificar permissões (apenas root)
        // - Verificar endereço válido
        // - CUIDADO: pode crashar o sistema!
        Err("mem write not implemented")
    }

    fn ioctl(&self, cmd: usize, _arg: usize) -> Result<usize, &'static str> {
        match cmd {
            // TODO: Implementar ioctls para mapeamento
            _ => Err("Unknown ioctl"),
        }
    }
}

impl CharDevice for MemDevice {}

/// /dev/kmem - Acesso à memória do kernel
pub struct KmemDevice {
    dev: DeviceNumber,
}

impl KmemDevice {
    /// Cria um novo /dev/kmem
    pub const fn new() -> Self {
        Self {
            dev: DeviceNumber::new(1, 2),
        }
    }
}

impl Device for KmemDevice {
    fn name(&self) -> &str {
        "kmem"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Character
    }

    fn device_number(&self) -> DeviceNumber {
        self.dev
    }

    fn read(&self, _buf: &mut [u8]) -> Result<usize, &'static str> {
        // TODO: Implementar leitura de memória do kernel
        Err("kmem read not implemented")
    }

    fn write(&self, _buf: &[u8]) -> Result<usize, &'static str> {
        // TODO: Implementar escrita em memória do kernel
        Err("kmem write not implemented")
    }
}

impl CharDevice for KmemDevice {}
