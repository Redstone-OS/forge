//! # Estruturas de Dados xHCI
//!
//! Transfer Request Blocks (TRBs), Device Contexts, Ring Structures.

#![allow(dead_code)]

/// Transfer Request Block (TRB) - 16 bytes
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default)]
pub struct Trb {
    /// Parameter low (usage depends on TRB type)
    pub param_lo: u32,
    /// Parameter high
    pub param_hi: u32,
    /// Status (completion code, transfer length, etc.)
    pub status: u32,
    /// Control (TRB type, cycle bit, flags)
    pub control: u32,
}

impl Trb {
    /// Cria um TRB zerado
    pub const fn new() -> Self {
        Self {
            param_lo: 0,
            param_hi: 0,
            status: 0,
            control: 0,
        }
    }

    /// Define o tipo do TRB
    pub fn set_type(&mut self, trb_type: u32) {
        self.control = (self.control & !0xFC00) | ((trb_type & 0x3F) << 10);
    }

    /// Obtém o tipo do TRB
    pub fn trb_type(&self) -> u32 {
        (self.control >> 10) & 0x3F
    }

    /// Define o cycle bit
    pub fn set_cycle(&mut self, cycle: bool) {
        if cycle {
            self.control |= 1;
        } else {
            self.control &= !1;
        }
    }

    /// Obtém o cycle bit
    pub fn cycle(&self) -> bool {
        (self.control & 1) != 0
    }

    /// Define o endereço de 64 bits no param
    pub fn set_param_ptr(&mut self, addr: u64) {
        self.param_lo = addr as u32;
        self.param_hi = (addr >> 32) as u32;
    }

    /// Obtém o endereço de 64 bits do param
    pub fn param_ptr(&self) -> u64 {
        (self.param_lo as u64) | ((self.param_hi as u64) << 32)
    }

    /// Obtém o completion code do status
    pub fn completion_code(&self) -> u8 {
        ((self.status >> 24) & 0xFF) as u8
    }

    /// Obtém o transfer length residual
    pub fn transfer_length(&self) -> u32 {
        self.status & 0xFFFFFF
    }

    /// Cria um Link TRB
    pub fn link(next_ring_addr: u64, toggle_cycle: bool) -> Self {
        let mut trb = Self::new();
        trb.set_param_ptr(next_ring_addr);
        trb.set_type(super::regs::trb_type::LINK);
        if toggle_cycle {
            trb.control |= 1 << 1; // Toggle Cycle bit
        }
        trb
    }

    /// Cria um Enable Slot Command
    pub fn enable_slot() -> Self {
        let mut trb = Self::new();
        trb.set_type(super::regs::trb_type::ENABLE_SLOT);
        trb
    }

    /// Cria um Address Device Command
    pub fn address_device(input_context_addr: u64, slot_id: u8, bsr: bool) -> Self {
        let mut trb = Self::new();
        trb.set_param_ptr(input_context_addr);
        trb.set_type(super::regs::trb_type::ADDRESS_DEVICE);
        trb.control |= (slot_id as u32) << 24;
        if bsr {
            trb.control |= 1 << 9; // Block Set Address Request
        }
        trb
    }

    /// Cria um No-Op Command
    pub fn noop() -> Self {
        let mut trb = Self::new();
        trb.set_type(super::regs::trb_type::NO_OP_CMD);
        trb
    }

    /// Cria um Setup Stage TRB (para Control transfers)
    pub fn setup_stage(request: SetupPacket, trt: u8) -> Self {
        let mut trb = Self::new();
        // Setup packet vai nos params
        let data = request.to_bytes();
        trb.param_lo = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        trb.param_hi = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        trb.status = 8; // TRB Transfer Length = 8 bytes
        trb.set_type(super::regs::trb_type::SETUP_STAGE);
        trb.control |= 1 << 6; // IDT (Immediate Data)
        trb.control |= (trt as u32 & 0x3) << 16; // Transfer Type
        trb
    }

    /// Cria um Data Stage TRB
    pub fn data_stage(data_addr: u64, length: u32, direction_in: bool) -> Self {
        let mut trb = Self::new();
        trb.set_param_ptr(data_addr);
        trb.status = length & 0x1FFFF;
        trb.set_type(super::regs::trb_type::DATA_STAGE);
        if direction_in {
            trb.control |= 1 << 16; // DIR = IN
        }
        trb
    }

    /// Cria um Status Stage TRB
    pub fn status_stage(direction_in: bool) -> Self {
        let mut trb = Self::new();
        trb.set_type(super::regs::trb_type::STATUS_STAGE);
        if direction_in {
            trb.control |= 1 << 16; // DIR = IN
        }
        trb.control |= 1 << 5; // IOC (Interrupt On Completion)
        trb
    }

    /// Cria um Normal TRB (para Bulk/Interrupt transfers)
    pub fn normal(data_addr: u64, length: u32, ioc: bool) -> Self {
        let mut trb = Self::new();
        trb.set_param_ptr(data_addr);
        trb.status = length & 0x1FFFF;
        trb.set_type(super::regs::trb_type::NORMAL);
        if ioc {
            trb.control |= 1 << 5; // IOC
        }
        trb
    }
}

/// USB Setup Packet (8 bytes)
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SetupPacket {
    /// Request type
    pub bm_request_type: u8,
    /// Request
    pub b_request: u8,
    /// Value
    pub w_value: u16,
    /// Index
    pub w_index: u16,
    /// Length
    pub w_length: u16,
}

impl SetupPacket {
    /// Converte para bytes
    pub fn to_bytes(&self) -> [u8; 8] {
        unsafe { core::mem::transmute_copy(self) }
    }

    /// GET_DESCRIPTOR request
    pub fn get_descriptor(desc_type: u8, desc_index: u8, length: u16) -> Self {
        Self {
            bm_request_type: 0x80, // Device to Host, Standard, Device
            b_request: 0x06,       // GET_DESCRIPTOR
            w_value: ((desc_type as u16) << 8) | (desc_index as u16),
            w_index: 0,
            w_length: length,
        }
    }

    /// SET_ADDRESS request
    pub fn set_address(address: u8) -> Self {
        Self {
            bm_request_type: 0x00, // Host to Device, Standard, Device
            b_request: 0x05,       // SET_ADDRESS
            w_value: address as u16,
            w_index: 0,
            w_length: 0,
        }
    }

    /// SET_CONFIGURATION request
    pub fn set_configuration(config: u8) -> Self {
        Self {
            bm_request_type: 0x00,
            b_request: 0x09, // SET_CONFIGURATION
            w_value: config as u16,
            w_index: 0,
            w_length: 0,
        }
    }

    /// GET_STATUS (Bulk-Only Mass Storage)
    pub fn bulk_only_reset(interface: u16) -> Self {
        Self {
            bm_request_type: 0x21, // Class, Interface, Host to Device
            b_request: 0xFF,       // Bulk-Only Mass Storage Reset
            w_value: 0,
            w_index: interface,
            w_length: 0,
        }
    }

    /// GET_MAX_LUN (Bulk-Only Mass Storage)
    pub fn get_max_lun(interface: u16) -> Self {
        Self {
            bm_request_type: 0xA1, // Class, Interface, Device to Host
            b_request: 0xFE,       // Get Max LUN
            w_value: 0,
            w_index: interface,
            w_length: 1,
        }
    }
}

/// Tipos de descriptor USB
pub mod descriptor_type {
    pub const DEVICE: u8 = 1;
    pub const CONFIGURATION: u8 = 2;
    pub const STRING: u8 = 3;
    pub const INTERFACE: u8 = 4;
    pub const ENDPOINT: u8 = 5;
    pub const DEVICE_QUALIFIER: u8 = 6;
    pub const OTHER_SPEED_CONFIG: u8 = 7;
    pub const INTERFACE_POWER: u8 = 8;
    pub const OTG: u8 = 9;
    pub const DEBUG: u8 = 10;
    pub const INTERFACE_ASSOCIATION: u8 = 11;
    pub const BOS: u8 = 15;
    pub const DEVICE_CAPABILITY: u8 = 16;
    pub const HID: u8 = 0x21;
    pub const HID_REPORT: u8 = 0x22;
    pub const HID_PHYSICAL: u8 = 0x23;
}

/// Device Descriptor (18 bytes)
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DeviceDescriptor {
    /// Size of this descriptor
    pub b_length: u8,
    /// Descriptor type (1 = Device)
    pub b_descriptor_type: u8,
    /// USB Specification version (BCD)
    pub bcd_usb: u16,
    /// Class code
    pub b_device_class: u8,
    /// Subclass code
    pub b_device_sub_class: u8,
    /// Protocol code
    pub b_device_protocol: u8,
    /// Max packet size for endpoint 0
    pub b_max_packet_size0: u8,
    /// Vendor ID
    pub id_vendor: u16,
    /// Product ID
    pub id_product: u16,
    /// Device release number (BCD)
    pub bcd_device: u16,
    /// Manufacturer string index
    pub i_manufacturer: u8,
    /// Product string index
    pub i_product: u8,
    /// Serial number string index
    pub i_serial_number: u8,
    /// Number of configurations
    pub b_num_configurations: u8,
}

/// Configuration Descriptor (9 bytes header)
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ConfigurationDescriptor {
    /// Size of this descriptor
    pub b_length: u8,
    /// Descriptor type (2 = Configuration)
    pub b_descriptor_type: u8,
    /// Total length of configuration data
    pub w_total_length: u16,
    /// Number of interfaces
    pub b_num_interfaces: u8,
    /// Configuration value
    pub b_configuration_value: u8,
    /// Configuration string index
    pub i_configuration: u8,
    /// Attributes (bit 7 = reserved, bit 6 = self-powered, bit 5 = remote wakeup)
    pub bm_attributes: u8,
    /// Max power (in 2mA units)
    pub b_max_power: u8,
}

/// Interface Descriptor (9 bytes)
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InterfaceDescriptor {
    /// Size of this descriptor
    pub b_length: u8,
    /// Descriptor type (4 = Interface)
    pub b_descriptor_type: u8,
    /// Interface number
    pub b_interface_number: u8,
    /// Alternate setting
    pub b_alternate_setting: u8,
    /// Number of endpoints
    pub b_num_endpoints: u8,
    /// Interface class
    pub b_interface_class: u8,
    /// Interface subclass
    pub b_interface_sub_class: u8,
    /// Interface protocol
    pub b_interface_protocol: u8,
    /// Interface string index
    pub i_interface: u8,
}

/// Endpoint Descriptor (7 bytes)
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct EndpointDescriptor {
    /// Size of this descriptor
    pub b_length: u8,
    /// Descriptor type (5 = Endpoint)
    pub b_descriptor_type: u8,
    /// Endpoint address (bit 7 = direction, bits 3:0 = endpoint number)
    pub b_endpoint_address: u8,
    /// Attributes (bits 1:0 = transfer type)
    pub bm_attributes: u8,
    /// Max packet size
    pub w_max_packet_size: u16,
    /// Polling interval (in frames/microframes)
    pub b_interval: u8,
}

impl EndpointDescriptor {
    /// Endpoint number (0-15)
    pub fn endpoint_number(&self) -> u8 {
        self.b_endpoint_address & 0x0F
    }

    /// Direction (true = IN, false = OUT)
    pub fn is_in(&self) -> bool {
        (self.b_endpoint_address & 0x80) != 0
    }

    /// Transfer type
    pub fn transfer_type(&self) -> EndpointType {
        match self.bm_attributes & 0x03 {
            0 => EndpointType::Control,
            1 => EndpointType::Isochronous,
            2 => EndpointType::Bulk,
            3 => EndpointType::Interrupt,
            _ => unreachable!(),
        }
    }
}

/// Tipo de endpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointType {
    Control = 0,
    Isochronous = 1,
    Bulk = 2,
    Interrupt = 3,
}

/// USB Classes
pub mod usb_class {
    pub const INTERFACE_SPECIFIC: u8 = 0x00;
    pub const AUDIO: u8 = 0x01;
    pub const CDC: u8 = 0x02;
    pub const HID: u8 = 0x03;
    pub const PHYSICAL: u8 = 0x05;
    pub const IMAGE: u8 = 0x06;
    pub const PRINTER: u8 = 0x07;
    pub const MASS_STORAGE: u8 = 0x08;
    pub const HUB: u8 = 0x09;
    pub const CDC_DATA: u8 = 0x0A;
    pub const SMART_CARD: u8 = 0x0B;
    pub const VIDEO: u8 = 0x0E;
    pub const AUDIO_VIDEO: u8 = 0x10;
    pub const WIRELESS: u8 = 0xE0;
    pub const VENDOR_SPECIFIC: u8 = 0xFF;
}

/// Mass Storage Subclass
pub mod mass_storage_subclass {
    pub const RBC: u8 = 0x01;
    pub const SFF8020I: u8 = 0x02; // ATAPI
    pub const QIC157: u8 = 0x03;
    pub const UFI: u8 = 0x04;
    pub const SFF8070I: u8 = 0x05;
    pub const SCSI: u8 = 0x06;
}

/// Mass Storage Protocol
pub mod mass_storage_protocol {
    pub const CBI_INTERRUPT: u8 = 0x00;
    pub const CBI_NO_INTERRUPT: u8 = 0x01;
    pub const BULK_ONLY: u8 = 0x50;
    pub const UAS: u8 = 0x62;
}

/// Slot Context (32 bytes - para CSZ=0)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SlotContext {
    /// Dword 0: Route String, Speed, MTT, Hub, Context Entries
    pub dword0: u32,
    /// Dword 1: Max Exit Latency, Root Hub Port Number, Number of Ports
    pub dword1: u32,
    /// Dword 2: Parent Hub Slot ID, Parent Port Number, TTT, Interrupter Target
    pub dword2: u32,
    /// Dword 3: USB Device Address, Slot State
    pub dword3: u32,
    /// Reserved (padding to 64 bytes para compatibilidade CSZ=1)
    pub reserved: [u32; 12],
}

/// Endpoint Context (32 bytes - para CSZ=0)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct EndpointContext {
    /// Dword 0: EP State, Mult, MaxPStreams, LSA, Interval, CErr
    pub dword0: u32,
    /// Dword 1: EP Type, HID, Max Burst Size, Max Packet Size
    pub dword1: u32,
    /// Dword 2-3: TR Dequeue Pointer (64-bit)
    pub tr_dequeue_lo: u32,
    pub tr_dequeue_hi: u32,
    /// Dword 4: Avg TRB Length, Max ESIT Payload low
    pub dword4: u32,
    /// Reserved (padding to 64 bytes para compatibilidade CSZ=1)
    pub reserved: [u32; 11],
}

/// Device Context (Slot Context + 31 Endpoint Contexts)
#[repr(C, align(4096))]
#[derive(Clone, Copy, Debug, Default)]
pub struct DeviceContext {
    pub slot: SlotContext,
    pub endpoints: [EndpointContext; 31],
}

/// Input Control Context (32 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InputControlContext {
    /// Drop Context Flags
    pub drop_flags: u32,
    /// Add Context Flags
    pub add_flags: u32,
    /// Reserved
    pub reserved: [u32; 5],
    /// Configuration Value, Interface Number, Alternate Setting
    pub dword7: u32,
}

/// Input Context (Input Control + Slot + Endpoints)
#[repr(C, align(4096))]
#[derive(Clone, Copy, Debug, Default)]
pub struct InputContext {
    pub control: InputControlContext,
    pub slot: SlotContext,
    pub endpoints: [EndpointContext; 31],
}

/// Event Ring Segment Table Entry
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, Default)]
pub struct ErstEntry {
    /// Ring Segment Base Address (64-bit, bits 63:6)
    pub base_lo: u32,
    pub base_hi: u32,
    /// Ring Segment Size (number of TRBs)
    pub size: u32,
    /// Reserved
    pub reserved: u32,
}

impl ErstEntry {
    pub const fn new() -> Self {
        Self {
            base_lo: 0,
            base_hi: 0,
            size: 0,
            reserved: 0,
        }
    }
}
