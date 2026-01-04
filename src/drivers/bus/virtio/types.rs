//! # VirtIO Types
//!
//! Constantes e enums universais para o padrão VirtIO.

/// IDs de dispositivos VirtIO conhecidos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtioDeviceId {
    NetworkCard = 1,
    BlockDevice = 2,
    Console = 3,
    EntropySource = 4,
    MemoryBalloon = 5,
    GpuDevice = 16,
    InputDevice = 18,
    SocketDevice = 19,
    Unknown(u32),
}

impl From<u32> for VirtioDeviceId {
    fn from(val: u32) -> Self {
        match val {
            1 => Self::NetworkCard,
            2 => Self::BlockDevice,
            3 => Self::Console,
            4 => Self::EntropySource,
            5 => Self::MemoryBalloon,
            16 => Self::GpuDevice,
            18 => Self::InputDevice,
            19 => Self::SocketDevice,
            other => Self::Unknown(other),
        }
    }
}

/// Status do dispositivo VirtIO (Device Status Field)
pub mod status {
    pub const ACKNOWLEDGE: u8 = 1;
    pub const DRIVER: u8 = 2;
    pub const DRIVER_OK: u8 = 4;
    pub const FEATURES_OK: u8 = 8;
    pub const DEVICE_NEEDS_RESET: u8 = 64;
    pub const FAILED: u8 = 128;
}

/// Flags de configuração comuns
pub mod flags {
    pub const VIRTIO_F_VERSION_1: u64 = 1 << 32;
    pub const VIRTIO_F_IOMMU_PLATFORM: u64 = 1 << 33;
    pub const VIRTIO_F_RING_PACKED: u64 = 1 << 34;
    pub const VIRTIO_F_IN_ORDER: u64 = 1 << 35;
}
