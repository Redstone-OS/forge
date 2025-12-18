//! Device - Trait e tipos base para dispositivos

use core::fmt;

/// Tipo de dispositivo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Dispositivo de caractere (char device)
    Character,
    /// Dispositivo de bloco (block device)
    Block,
}

/// Número major/minor de dispositivo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceNumber {
    /// Major number (identifica o driver)
    pub major: u32,
    /// Minor number (identifica o dispositivo específico)
    pub minor: u32,
}

impl DeviceNumber {
    /// Cria um novo device number
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Converte para u64 (formato Linux: major << 20 | minor)
    pub const fn as_u64(&self) -> u64 {
        ((self.major as u64) << 20) | (self.minor as u64)
    }

    /// Cria a partir de u64
    pub const fn from_u64(dev: u64) -> Self {
        Self {
            major: (dev >> 20) as u32,
            minor: (dev & 0xFFFFF) as u32,
        }
    }
}

/// Trait para dispositivos
pub trait Device: Send + Sync {
    /// Retorna o nome do dispositivo
    fn name(&self) -> &str;

    /// Retorna o tipo de dispositivo
    fn device_type(&self) -> DeviceType;

    /// Retorna o device number
    fn device_number(&self) -> DeviceNumber;

    /// Abre o dispositivo
    fn open(&self) -> Result<(), &'static str> {
        Ok(())
    }

    /// Fecha o dispositivo
    fn close(&self) -> Result<(), &'static str> {
        Ok(())
    }

    /// Lê do dispositivo
    fn read(&self, _buf: &mut [u8]) -> Result<usize, &'static str> {
        Err("Read not supported")
    }

    /// Escreve no dispositivo
    fn write(&self, _buf: &[u8]) -> Result<usize, &'static str> {
        Err("Write not supported")
    }

    /// ioctl (controle de dispositivo)
    fn ioctl(&self, _cmd: usize, _arg: usize) -> Result<usize, &'static str> {
        Err("ioctl not supported")
    }
}

/// Nó de dispositivo
pub struct DeviceNode {
    /// Nome do dispositivo
    pub name: &'static str,
    /// Tipo de dispositivo
    pub device_type: DeviceType,
    /// Device number
    pub dev: DeviceNumber,
    /// Permissões (Unix mode)
    pub mode: u16,
    /// UID do dono
    pub uid: u32,
    /// GID do grupo
    pub gid: u32,
}

impl DeviceNode {
    /// Cria um novo device node
    pub const fn new(name: &'static str, device_type: DeviceType, major: u32, minor: u32) -> Self {
        Self {
            name,
            device_type,
            dev: DeviceNumber::new(major, minor),
            mode: 0o666, // rw-rw-rw- por padrão
            uid: 0,      // root
            gid: 0,      // root
        }
    }

    /// Cria um device node com permissões customizadas
    pub const fn with_mode(mut self, mode: u16) -> Self {
        self.mode = mode;
        self
    }
}

impl fmt::Debug for DeviceNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceNode")
            .field("name", &self.name)
            .field("type", &self.device_type)
            .field("major", &self.dev.major)
            .field("minor", &self.dev.minor)
            .field("mode", &format_args!("{:o}", self.mode))
            .finish()
    }
}
