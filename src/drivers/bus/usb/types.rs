//! # USB Types and Constants
//!
//! Constantes e tipos fundamentais do protocolo USB.

// =============================================================================
// VELOCIDADES USB
// =============================================================================

/// Velocidade de conexão USB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    /// Low Speed: 1.5 Mbps (USB 1.0)
    Low = 0,
    /// Full Speed: 12 Mbps (USB 1.1)
    Full = 1,
    /// High Speed: 480 Mbps (USB 2.0)
    High = 2,
    /// SuperSpeed: 5 Gbps (USB 3.0)
    Super = 3,
    /// SuperSpeed+: 10 Gbps (USB 3.1)
    SuperPlus = 4,
    /// SuperSpeed+ 20G: 20 Gbps (USB 3.2)
    SuperPlus20 = 5,
}

impl UsbSpeed {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "Low Speed (1.5 Mbps)",
            Self::Full => "Full Speed (12 Mbps)",
            Self::High => "High Speed (480 Mbps)",
            Self::Super => "SuperSpeed (5 Gbps)",
            Self::SuperPlus => "SuperSpeed+ (10 Gbps)",
            Self::SuperPlus20 => "SuperSpeed+ (20 Gbps)",
        }
    }
}

// =============================================================================
// TIPOS DE TRANSFER
// =============================================================================

/// Tipo de transferência USB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbTransferType {
    /// Controle: Setup, Data, Status phases
    Control,
    /// Bulk: Grandes quantidades de dados, sem garantia de tempo
    Bulk,
    /// Interrupt: Pequenos dados, polling periódico
    Interrupt,
    /// Isochronous: Streaming em tempo real, sem retransmissão
    Isochronous,
}

// =============================================================================
// DIREÇÃO DE TRANSFER
// =============================================================================

/// Direção de uma transferência.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDirection {
    /// Host → Device (OUT)
    Out = 0,
    /// Device → Host (IN)
    In = 1,
}

// =============================================================================
// CLASSES USB
// =============================================================================

/// Classes de dispositivo USB padrão.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbClass {
    /// Definido por interface
    PerInterface = 0x00,
    /// Audio
    Audio = 0x01,
    /// CDC (Communications Device Class)
    Cdc = 0x02,
    /// HID (Human Interface Device)
    Hid = 0x03,
    /// Physical
    Physical = 0x05,
    /// Image
    Image = 0x06,
    /// Printer
    Printer = 0x07,
    /// Mass Storage
    MassStorage = 0x08,
    /// Hub
    Hub = 0x09,
    /// CDC Data
    CdcData = 0x0A,
    /// Smart Card
    SmartCard = 0x0B,
    /// Content Security
    ContentSecurity = 0x0D,
    /// Video
    Video = 0x0E,
    /// Personal Healthcare
    Healthcare = 0x0F,
    /// Audio/Video
    AudioVideo = 0x10,
    /// Billboard
    Billboard = 0x11,
    /// USB-C Bridge
    UsbcBridge = 0x12,
    /// Diagnostic
    Diagnostic = 0xDC,
    /// Wireless Controller
    Wireless = 0xE0,
    /// Miscellaneous
    Miscellaneous = 0xEF,
    /// Application Specific
    ApplicationSpecific = 0xFE,
    /// Vendor Specific
    VendorSpecific = 0xFF,
}

// =============================================================================
// SETUP PACKET
// =============================================================================

/// USB Setup Packet - 8 bytes usados em Control Transfers.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct UsbSetupPacket {
    /// Request type (direction, type, recipient)
    pub request_type: u8,
    /// Request code
    pub request: u8,
    /// Value (request-specific)
    pub value: u16,
    /// Index (request-specific)
    pub index: u16,
    /// Length of data stage
    pub length: u16,
}

impl UsbSetupPacket {
    /// Cria um GET_DESCRIPTOR request.
    pub fn get_descriptor(desc_type: u8, desc_index: u8, lang_id: u16, length: u16) -> Self {
        Self {
            request_type: 0x80, // Device-to-host, Standard, Device
            request: USB_REQ_GET_DESCRIPTOR,
            value: ((desc_type as u16) << 8) | (desc_index as u16),
            index: lang_id,
            length,
        }
    }

    /// Cria um SET_ADDRESS request.
    pub fn set_address(address: u8) -> Self {
        Self {
            request_type: 0x00, // Host-to-device, Standard, Device
            request: USB_REQ_SET_ADDRESS,
            value: address as u16,
            index: 0,
            length: 0,
        }
    }

    /// Cria um SET_CONFIGURATION request.
    pub fn set_configuration(config_value: u8) -> Self {
        Self {
            request_type: 0x00,
            request: USB_REQ_SET_CONFIGURATION,
            value: config_value as u16,
            index: 0,
            length: 0,
        }
    }
}

// =============================================================================
// STANDARD REQUESTS
// =============================================================================

pub const USB_REQ_GET_STATUS: u8 = 0;
pub const USB_REQ_CLEAR_FEATURE: u8 = 1;
pub const USB_REQ_SET_FEATURE: u8 = 3;
pub const USB_REQ_SET_ADDRESS: u8 = 5;
pub const USB_REQ_GET_DESCRIPTOR: u8 = 6;
pub const USB_REQ_SET_DESCRIPTOR: u8 = 7;
pub const USB_REQ_GET_CONFIGURATION: u8 = 8;
pub const USB_REQ_SET_CONFIGURATION: u8 = 9;
pub const USB_REQ_GET_INTERFACE: u8 = 10;
pub const USB_REQ_SET_INTERFACE: u8 = 11;
pub const USB_REQ_SYNCH_FRAME: u8 = 12;

// =============================================================================
// DESCRIPTOR TYPES
// =============================================================================

pub const USB_DESC_TYPE_DEVICE: u8 = 1;
pub const USB_DESC_TYPE_CONFIGURATION: u8 = 2;
pub const USB_DESC_TYPE_STRING: u8 = 3;
pub const USB_DESC_TYPE_INTERFACE: u8 = 4;
pub const USB_DESC_TYPE_ENDPOINT: u8 = 5;
pub const USB_DESC_TYPE_DEVICE_QUALIFIER: u8 = 6;
pub const USB_DESC_TYPE_OTHER_SPEED_CONFIG: u8 = 7;
pub const USB_DESC_TYPE_INTERFACE_POWER: u8 = 8;
pub const USB_DESC_TYPE_OTG: u8 = 9;
pub const USB_DESC_TYPE_DEBUG: u8 = 10;
pub const USB_DESC_TYPE_INTERFACE_ASSOCIATION: u8 = 11;
pub const USB_DESC_TYPE_BOS: u8 = 15;
pub const USB_DESC_TYPE_DEVICE_CAPABILITY: u8 = 16;
pub const USB_DESC_TYPE_HID: u8 = 33;
pub const USB_DESC_TYPE_REPORT: u8 = 34;
pub const USB_DESC_TYPE_PHYSICAL: u8 = 35;
pub const USB_DESC_TYPE_HUB: u8 = 41;
pub const USB_DESC_TYPE_SUPERSPEED_HUB: u8 = 42;
pub const USB_DESC_TYPE_SS_ENDPOINT_COMPANION: u8 = 48;

// =============================================================================
// DESCRIPTORS
// =============================================================================

/// Device Descriptor (18 bytes)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct UsbDeviceDescriptor {
    pub length: u8,          // 18
    pub descriptor_type: u8, // 1
    pub bcd_usb: u16,        // USB version (BCD)
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size_0: u8,
    pub id_vendor: u16,
    pub id_product: u16,
    pub bcd_device: u16, // Device version (BCD)
    pub i_manufacturer: u8,
    pub i_product: u8,
    pub i_serial_number: u8,
    pub num_configurations: u8,
}

/// Configuration Descriptor Header (9 bytes)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct UsbConfigurationDescriptor {
    pub length: u8,          // 9
    pub descriptor_type: u8, // 2
    pub total_length: u16,   // Total length including interfaces/endpoints
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub i_configuration: u8,
    pub attributes: u8,
    pub max_power: u8, // In 2mA units
}

/// Interface Descriptor (9 bytes)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct UsbInterfaceDescriptor {
    pub length: u8,          // 9
    pub descriptor_type: u8, // 4
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub i_interface: u8,
}

/// Endpoint Descriptor (7 bytes)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct UsbEndpointDescriptor {
    pub length: u8,           // 7
    pub descriptor_type: u8,  // 5
    pub endpoint_address: u8, // Bit 7 = direction, bits 3:0 = endpoint number
    pub attributes: u8,       // Bits 1:0 = transfer type
    pub max_packet_size: u16,
    pub interval: u8, // Polling interval
}

impl UsbEndpointDescriptor {
    /// Retorna o número do endpoint.
    pub fn endpoint_number(&self) -> u8 {
        self.endpoint_address & 0x0F
    }

    /// Retorna a direção.
    pub fn direction(&self) -> UsbDirection {
        if self.endpoint_address & 0x80 != 0 {
            UsbDirection::In
        } else {
            UsbDirection::Out
        }
    }

    /// Retorna o tipo de transferência.
    pub fn transfer_type(&self) -> UsbTransferType {
        match self.attributes & 0x03 {
            0 => UsbTransferType::Control,
            1 => UsbTransferType::Isochronous,
            2 => UsbTransferType::Bulk,
            3 => UsbTransferType::Interrupt,
            _ => unreachable!(),
        }
    }
}
