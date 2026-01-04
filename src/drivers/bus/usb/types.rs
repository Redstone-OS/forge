//! # USB Common Types
//!
//! Tipos e enums universais para o subsistema USB.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    FullSpeed,      // 12 Mbps (USB 1.1)
    LowSpeed,       // 1.5 Mbps (USB 1.1)
    HighSpeed,      // 480 Mbps (USB 2.0)
    SuperSpeed,     // 5 Gbps (USB 3.0)
    SuperSpeedPlus, // 10+ Gbps (USB 3.1+)
}

impl UsbSpeed {
    pub fn name(&self) -> &'static str {
        match self {
            Self::FullSpeed => "FullSpeed (12Mbps)",
            Self::LowSpeed => "LowSpeed (1.5Mbps)",
            Self::HighSpeed => "HighSpeed (480Mbps)",
            Self::SuperSpeed => "SuperSpeed (5Gbps)",
            Self::SuperSpeedPlus => "SuperSpeedPlus (10Gbps+)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorType {
    Device = 1,
    Configuration = 2,
    String = 3,
    Interface = 4,
    Endpoint = 5,
    Hub = 0x29,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbDeviceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub bcd_usb: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size0: u8,
    pub id_vendor: u16,
    pub id_product: u16,
    pub bcd_device: u16,
    pub manufacturer_idx: u8,
    pub product_idx: u8,
    pub serial_number_idx: u8,
    pub num_configurations: u8,
}
