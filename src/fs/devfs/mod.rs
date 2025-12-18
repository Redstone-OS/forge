//! DevFS - Device Filesystem
//!
//! Sistema de arquivos para dispositivos (/dev).
//!
//! # Arquitetura
//!
//! ## Dispositivos Kernel-Space (Ring 0)
//! - **Essenciais:** null, zero, console, mem, rtc
//! - **Bloco:** sda, nvme (boot crítico)
//! - **TTY:** tty0, ttyS0 (console/serial)
//!
//! ## Dispositivos Userspace (Ring 3)
//! - **USB:** Complexo, userspace
//! - **Áudio:** snd/* (ALSA userspace)
//! - **Rede:** Híbrido (kernel NIC, userspace TCP/IP)
//!
//! # Módulos
//!
//! - `device` - Trait Device e tipos base
//! - `char_device` - Dispositivos de caractere
//! - `block_device` - Dispositivos de bloco
//! - `registry` - Registro global de dispositivos
//! - `operations` - Operações (read, write, ioctl)
//! - `devices/*` - Implementações específicas

#![allow(dead_code)]

pub mod block_device;
pub mod char_device;
pub mod device;
pub mod operations;
pub mod registry;

// Dispositivos implementados (essenciais)
pub mod devices {
    pub mod console;
    pub mod mem;
    pub mod null;
    pub mod rtc;
    pub mod tty;
    pub mod zero;

    // Dispositivos opcionais (TODOs)
    pub mod fb;
    pub mod input;
    pub mod net;
    pub mod random;
    pub mod snd;
    pub mod usb;
}

// Re-exports públicos
pub use block_device::BlockDevice;
pub use char_device::CharDevice;
pub use device::{Device, DeviceNumber, DeviceType};
pub use operations::{DeviceOps, OpenFlags};
pub use registry::DeviceRegistry;

/// DevFS - Device Filesystem
pub struct DevFS {
    /// Registro de dispositivos
    registry: DeviceRegistry,
}

impl DevFS {
    /// Cria uma nova instância de DevFS
    pub fn new() -> Self {
        let mut devfs = Self {
            registry: DeviceRegistry::new(),
        };

        // Registra dispositivos essenciais
        devfs.register_essential_devices();

        devfs
    }

    /// Registra dispositivos essenciais do kernel
    fn register_essential_devices(&mut self) {
        // TODO: Registrar dispositivos quando implementados
        // self.registry.register(devices::null::NullDevice::new());
        // self.registry.register(devices::zero::ZeroDevice::new());
        // self.registry.register(devices::console::ConsoleDevice::new());
    }

    /// Abre um dispositivo por caminho
    pub fn open(&self, path: &str, flags: OpenFlags) -> Result<usize, &'static str> {
        self.registry.open(path, flags)
    }

    /// Fecha um dispositivo
    pub fn close(&self, fd: usize) -> Result<(), &'static str> {
        self.registry.close(fd)
    }

    /// Lê de um dispositivo
    pub fn read(&self, fd: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
        self.registry.read(fd, buf)
    }

    /// Escreve em um dispositivo
    pub fn write(&self, fd: usize, buf: &[u8]) -> Result<usize, &'static str> {
        self.registry.write(fd, buf)
    }
}

impl Default for DevFS {
    fn default() -> Self {
        Self::new()
    }
}

// Constantes de dispositivos (major/minor numbers do Linux)
// Referência: https://www.kernel.org/doc/Documentation/admin-guide/devices.txt

/// /dev/null - descarta tudo
pub const DEV_NULL: DeviceNumber = DeviceNumber::new(1, 3);

/// /dev/zero - retorna zeros
pub const DEV_ZERO: DeviceNumber = DeviceNumber::new(1, 5);

/// /dev/random - gerador aleatório (blocking)
pub const DEV_RANDOM: DeviceNumber = DeviceNumber::new(1, 8);

/// /dev/urandom - gerador aleatório (non-blocking)
pub const DEV_URANDOM: DeviceNumber = DeviceNumber::new(1, 9);

/// /dev/mem - memória física
pub const DEV_MEM: DeviceNumber = DeviceNumber::new(1, 1);

/// /dev/kmem - memória do kernel
pub const DEV_KMEM: DeviceNumber = DeviceNumber::new(1, 2);

/// /dev/tty - terminal atual
pub const DEV_TTY: DeviceNumber = DeviceNumber::new(5, 0);

/// /dev/console - console do sistema
pub const DEV_CONSOLE: DeviceNumber = DeviceNumber::new(5, 1);

/// /dev/ttyS0 - serial port 0
pub const DEV_TTYS0: DeviceNumber = DeviceNumber::new(4, 64);

/// /dev/rtc - relógio de tempo real
pub const DEV_RTC: DeviceNumber = DeviceNumber::new(10, 135);

/// /dev/fb0 - framebuffer 0
pub const DEV_FB0: DeviceNumber = DeviceNumber::new(29, 0);
