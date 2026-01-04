use super::ring::Ring;
pub use super::structs::DeviceDescriptor;

/// Informações de uma porta USB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbPort {
    /// Número da porta (1-based)
    pub port_num: u8,
    /// Se um dispositivo está fisicamente conectado
    pub connected: bool,
    /// Se a porta está habilitada (PED set)
    pub enabled: bool,
    /// Velocidade detectada
    pub speed: UsbSpeed,
}

/// Informações de um dispositivo USB conectado
pub struct UsbDevice {
    /// Slot ID atribuído pelo xHCI
    pub slot_id: u8,
    /// Porta onde está conectado
    pub port: u8,
    /// Velocidade
    pub speed: UsbSpeed,
    /// Device Descriptor
    pub device_desc: Option<DeviceDescriptor>,
    /// Vendor ID
    pub vendor_id: u16,
    /// Product ID
    pub product_id: u16,
    /// Device Class
    pub device_class: u8,
    /// Anéis de transferência para cada endpoint (índice = Context Index)
    pub rings: [Option<Ring>; 32],
}

impl core::fmt::Debug for UsbDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UsbDevice")
            .field("slot_id", &self.slot_id)
            .field("port", &self.port)
            .field("speed", &self.speed)
            .field("device_class", &self.device_class)
            .finish()
    }
}

impl UsbDevice {
    pub fn new(slot_id: u8, port: u8, speed: UsbSpeed) -> Self {
        const NONE_RING: Option<Ring> = None;
        Self {
            slot_id,
            port,
            speed,
            device_desc: None,
            vendor_id: 0,
            product_id: 0,
            device_class: 0,
            rings: [NONE_RING; 32],
        }
    }
}

/// Velocidade USB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    Full = 1,
    Low = 2,
    High = 3,
    Super = 4,
    SuperPlus = 5,
    Unknown = 0,
}

impl From<u32> for UsbSpeed {
    fn from(val: u32) -> Self {
        match val {
            1 => UsbSpeed::Full,
            2 => UsbSpeed::Low,
            3 => UsbSpeed::High,
            4 => UsbSpeed::Super,
            5 => UsbSpeed::SuperPlus,
            _ => UsbSpeed::Unknown,
        }
    }
}
