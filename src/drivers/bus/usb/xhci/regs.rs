//! # Constantes e Registradores xHCI
//!
//! Definições do eXtensible Host Controller Interface (USB 3.0).
//!
//! ## Referência
//! - Intel xHCI Specification 1.2
//! - OSDev Wiki: https://wiki.osdev.org/XHCI

#![allow(dead_code)]

// =============================================================================
// PCI IDs
// =============================================================================

/// PCI Class Code para Serial Bus Controller
pub const PCI_CLASS_SERIAL_BUS: u8 = 0x0C;

/// PCI Subclass para USB Controller
pub const PCI_SUBCLASS_USB: u8 = 0x03;

/// PCI Programming Interface para xHCI
pub const PCI_PROG_IF_XHCI: u8 = 0x30;

// =============================================================================
// CAPABILITY REGISTERS (Offset 0x00)
// =============================================================================

/// Offsets dos registradores de capability
pub mod cap {
    /// Capability Registers Length + HC Interface Version (32 bits)
    /// Bits 7:0 = CAPLENGTH, Bits 31:16 = HCIVERSION
    pub const CAPLENGTH_HCIVERSION: u64 = 0x00;

    /// Structural Parameters 1 (32 bits)
    /// MaxSlots (7:0), MaxIntrs (18:8), MaxPorts (31:24)
    pub const HCSPARAMS1: u64 = 0x04;

    /// Structural Parameters 2 (32 bits)
    /// IST (3:0), ERST Max (7:4), SPR (26), Max Scratchpad Bufs (31:27, 25:21)
    pub const HCSPARAMS2: u64 = 0x08;

    /// Structural Parameters 3 (32 bits)
    /// U1 Device Exit Latency (7:0), U2 Device Exit Latency (31:16)
    pub const HCSPARAMS3: u64 = 0x0C;

    /// Capability Parameters 1 (32 bits)
    /// AC64 (0), BNC (1), CSZ (2), PPC (3), PIND (4), LHRC (5), LTC (6), NSS (7)
    /// MaxPSASize (15:12), xECP (31:16)
    pub const HCCPARAMS1: u64 = 0x10;

    /// Doorbell Offset (32 bits) - alinhado a 4 bytes
    pub const DBOFF: u64 = 0x14;

    /// Runtime Register Space Offset (32 bits) - alinhado a 32 bytes
    pub const RTSOFF: u64 = 0x18;

    /// Capability Parameters 2 (32 bits)
    pub const HCCPARAMS2: u64 = 0x1C;
}

/// Bits do HCSPARAMS1
pub mod hcsparams1 {
    /// Máximo de Device Slots (bits 7:0)
    pub const MAX_SLOTS_MASK: u32 = 0xFF;
    /// Máximo de Interrupters (bits 18:8)
    pub const MAX_INTRS_SHIFT: u32 = 8;
    pub const MAX_INTRS_MASK: u32 = 0x7FF << MAX_INTRS_SHIFT;
    /// Máximo de Ports (bits 31:24)
    pub const MAX_PORTS_SHIFT: u32 = 24;
    pub const MAX_PORTS_MASK: u32 = 0xFF << MAX_PORTS_SHIFT;
}

/// Bits do HCSPARAMS2
pub mod hcsparams2 {
    /// Max Scratchpad Buffers Low (bits 25:21)
    pub const MAX_SCRATCHPAD_BUFS_LO_MASK: u32 = 0x1F << 21;
    pub const MAX_SCRATCHPAD_BUFS_LO_SHIFT: u32 = 21;
    /// Max Scratchpad Buffers High (bits 31:27)
    pub const MAX_SCRATCHPAD_BUFS_HI_MASK: u32 = 0x1F << 27;
    pub const MAX_SCRATCHPAD_BUFS_HI_SHIFT: u32 = 27;
}

/// Bits do HCCPARAMS1
pub mod hccparams1 {
    /// Suporta endereços de 64-bit
    pub const AC64: u32 = 1 << 0;
    /// Bandwidth Negotiation Capability
    pub const BNC: u32 = 1 << 1;
    /// Context Size (0 = 32 bytes, 1 = 64 bytes)
    pub const CSZ: u32 = 1 << 2;
    /// Port Power Control
    pub const PPC: u32 = 1 << 3;
    /// Port Indicators
    pub const PIND: u32 = 1 << 4;
    /// Light HC Reset Capability
    pub const LHRC: u32 = 1 << 5;
    /// Latency Tolerance Messaging Capability
    pub const LTC: u32 = 1 << 6;
    /// No Secondary SID Support
    pub const NSS: u32 = 1 << 7;
    /// Extended Capabilities Pointer (bits 31:16)
    pub const XECP_SHIFT: u32 = 16;
    pub const XECP_MASK: u32 = 0xFFFF << XECP_SHIFT;
}

// =============================================================================
// OPERATIONAL REGISTERS (Offset CAPLENGTH)
// =============================================================================

/// Offsets dos registradores operacionais (relativos a CAPLENGTH)
pub mod op {
    /// USB Command Register
    pub const USBCMD: u64 = 0x00;
    /// USB Status Register
    pub const USBSTS: u64 = 0x04;
    /// Page Size Register
    pub const PAGESIZE: u64 = 0x08;
    /// Device Notification Control Register
    pub const DNCTRL: u64 = 0x14;
    /// Command Ring Control Register (64-bit)
    pub const CRCR: u64 = 0x18;
    /// Device Context Base Address Array Pointer (64-bit)
    pub const DCBAAP: u64 = 0x30;
    /// Configure Register
    pub const CONFIG: u64 = 0x38;
}

/// Bits do USBCMD
pub mod usbcmd {
    /// Run/Stop (0 = Stop, 1 = Run)
    pub const RS: u32 = 1 << 0;
    /// Host Controller Reset
    pub const HCRST: u32 = 1 << 1;
    /// Interrupter Enable
    pub const INTE: u32 = 1 << 2;
    /// Host System Error Enable
    pub const HSEE: u32 = 1 << 3;
    /// Light Host Controller Reset
    pub const LHCRST: u32 = 1 << 7;
    /// Controller Save State
    pub const CSS: u32 = 1 << 8;
    /// Controller Restore State
    pub const CRS: u32 = 1 << 9;
    /// Enable Wrap Event
    pub const EWE: u32 = 1 << 10;
    /// Enable U3 MFINDEX Stop
    pub const EU3S: u32 = 1 << 11;
}

/// Bits do USBSTS
pub mod usbsts {
    /// HC Halted
    pub const HCH: u32 = 1 << 0;
    /// Host System Error
    pub const HSE: u32 = 1 << 2;
    /// Event Interrupt
    pub const EINT: u32 = 1 << 3;
    /// Port Change Detect
    pub const PCD: u32 = 1 << 4;
    /// Save State Status
    pub const SSS: u32 = 1 << 8;
    /// Restore State Status
    pub const RSS: u32 = 1 << 9;
    /// Save/Restore Error
    pub const SRE: u32 = 1 << 10;
    /// Controller Not Ready
    pub const CNR: u32 = 1 << 11;
    /// Host Controller Error
    pub const HCE: u32 = 1 << 12;
}

/// Bits do CRCR (Command Ring Control Register)
pub mod crcr {
    /// Ring Cycle State
    pub const RCS: u64 = 1 << 0;
    /// Command Stop
    pub const CS: u64 = 1 << 1;
    /// Command Abort
    pub const CA: u64 = 1 << 2;
    /// Command Ring Running
    pub const CRR: u64 = 1 << 3;
    /// Command Ring Pointer (bits 63:6) - deve ser alinhado a 64 bytes
    pub const CRP_MASK: u64 = !0x3F;
}

// =============================================================================
// PORT REGISTERS (Offset CAPLENGTH + 0x400 + 0x10 * (port_num - 1))
// =============================================================================

/// Offsets dos registradores de porta (relativos ao início das port registers)
pub mod port {
    /// Port Status and Control
    pub const PORTSC: u64 = 0x00;
    /// Port PM Status and Control
    pub const PORTPMSC: u64 = 0x04;
    /// Port Link Info
    pub const PORTLI: u64 = 0x08;
    /// Port Hardware LPM Control
    pub const PORTHLPMC: u64 = 0x0C;
}

/// Offset base das port registers (relativo a operational registers)
pub const PORT_REG_BASE: u64 = 0x400;

/// Tamanho de cada bloco de registradores de porta
pub const PORT_REG_SIZE: u64 = 0x10;

/// Bits do PORTSC
pub mod portsc {
    /// Current Connect Status
    pub const CCS: u32 = 1 << 0;
    /// Port Enabled/Disabled
    pub const PED: u32 = 1 << 1;
    /// Over-current Active
    pub const OCA: u32 = 1 << 3;
    /// Port Reset
    pub const PR: u32 = 1 << 4;
    /// Port Link State (bits 8:5)
    pub const PLS_SHIFT: u32 = 5;
    pub const PLS_MASK: u32 = 0xF << PLS_SHIFT;
    /// Port Power
    pub const PP: u32 = 1 << 9;
    /// Port Speed (bits 13:10)
    pub const PORT_SPEED_SHIFT: u32 = 10;
    pub const PORT_SPEED_MASK: u32 = 0xF << PORT_SPEED_SHIFT;
    /// Port Indicator Control (bits 15:14)
    pub const PIC_SHIFT: u32 = 14;
    pub const PIC_MASK: u32 = 0x3 << PIC_SHIFT;
    /// Port Link State Write Strobe
    pub const LWS: u32 = 1 << 16;
    /// Connect Status Change
    pub const CSC: u32 = 1 << 17;
    /// Port Enabled/Disabled Change
    pub const PEC: u32 = 1 << 18;
    /// Warm Port Reset Change
    pub const WRC: u32 = 1 << 19;
    /// Over-current Change
    pub const OCC: u32 = 1 << 20;
    /// Port Reset Change
    pub const PRC: u32 = 1 << 21;
    /// Port Link State Change
    pub const PLC: u32 = 1 << 22;
    /// Port Config Error Change
    pub const CEC: u32 = 1 << 23;
    /// Cold Attach Status
    pub const CAS: u32 = 1 << 24;
    /// Wake on Connect Enable
    pub const WCE: u32 = 1 << 25;
    /// Wake on Disconnect Enable
    pub const WDE: u32 = 1 << 26;
    /// Wake on Over-current Enable
    pub const WOE: u32 = 1 << 27;
    /// Device Removable
    pub const DR: u32 = 1 << 30;
    /// Warm Port Reset
    pub const WPR: u32 = 1 << 31;
}

/// Port Link State values
pub mod pls {
    pub const U0: u32 = 0; // On (USB 3), L0 (USB 2)
    pub const U1: u32 = 1;
    pub const U2: u32 = 2;
    pub const U3: u32 = 3; // Suspended
    pub const DISABLED: u32 = 4;
    pub const RX_DETECT: u32 = 5;
    pub const INACTIVE: u32 = 6;
    pub const POLLING: u32 = 7;
    pub const RECOVERY: u32 = 8;
    pub const HOT_RESET: u32 = 9;
    pub const COMPLIANCE_MODE: u32 = 10;
    pub const TEST_MODE: u32 = 11;
    pub const RESUME: u32 = 15;
}

/// Port Speed values
pub mod port_speed {
    pub const FULL: u32 = 1; // Full Speed (12 Mbps)
    pub const LOW: u32 = 2; // Low Speed (1.5 Mbps)
    pub const HIGH: u32 = 3; // High Speed (480 Mbps)
    pub const SUPER: u32 = 4; // SuperSpeed (5 Gbps)
    pub const SUPER_PLUS: u32 = 5; // SuperSpeed+ (10 Gbps)
}

// =============================================================================
// RUNTIME REGISTERS (Offset RTSOFF)
// =============================================================================

/// Offsets dos registradores de runtime (relativos a RTSOFF)
pub mod runtime {
    /// Microframe Index Register
    pub const MFINDEX: u64 = 0x00;
    /// Interrupter Register Set 0 (cada set tem 32 bytes)
    pub const IR0: u64 = 0x20;
}

/// Offset dentro de cada Interrupter Register Set
pub mod interrupter {
    /// Interrupter Management Register
    pub const IMAN: u64 = 0x00;
    /// Interrupter Moderation Register
    pub const IMOD: u64 = 0x04;
    /// Event Ring Segment Table Size
    pub const ERSTSZ: u64 = 0x08;
    /// Event Ring Segment Table Base Address (64-bit)
    pub const ERSTBA: u64 = 0x10;
    /// Event Ring Dequeue Pointer (64-bit)
    pub const ERDP: u64 = 0x18;
}

/// Tamanho de cada Interrupter Register Set
pub const INTERRUPTER_SIZE: u64 = 0x20;

// =============================================================================
// DOORBELL REGISTERS (Offset DBOFF)
// =============================================================================

/// Doorbell register para o Host Controller (slot 0)
pub const DB_HC: u32 = 0;

/// Macro para calcular offset do doorbell de um slot
pub fn doorbell_offset(slot_id: u8) -> u64 {
    (slot_id as u64) * 4
}

// =============================================================================
// TRANSFER REQUEST BLOCKS (TRBs)
// =============================================================================

/// Tamanho de um TRB em bytes
pub const TRB_SIZE: usize = 16;

/// Tipos de TRB (bits 15:10 do Control field)
pub mod trb_type {
    // Transfer TRBs
    pub const NORMAL: u32 = 1;
    pub const SETUP_STAGE: u32 = 2;
    pub const DATA_STAGE: u32 = 3;
    pub const STATUS_STAGE: u32 = 4;
    pub const ISOCH: u32 = 5;
    pub const LINK: u32 = 6;
    pub const EVENT_DATA: u32 = 7;
    pub const NO_OP: u32 = 8;

    // Command TRBs
    pub const ENABLE_SLOT: u32 = 9;
    pub const DISABLE_SLOT: u32 = 10;
    pub const ADDRESS_DEVICE: u32 = 11;
    pub const CONFIGURE_ENDPOINT: u32 = 12;
    pub const EVALUATE_CONTEXT: u32 = 13;
    pub const RESET_ENDPOINT: u32 = 14;
    pub const STOP_ENDPOINT: u32 = 15;
    pub const SET_TR_DEQUEUE: u32 = 16;
    pub const RESET_DEVICE: u32 = 17;
    pub const FORCE_EVENT: u32 = 18;
    pub const NEGOTIATE_BW: u32 = 19;
    pub const SET_LATENCY: u32 = 20;
    pub const GET_PORT_BW: u32 = 21;
    pub const FORCE_HEADER: u32 = 22;
    pub const NO_OP_CMD: u32 = 23;

    // Event TRBs
    pub const TRANSFER_EVENT: u32 = 32;
    pub const COMMAND_COMPLETION: u32 = 33;
    pub const PORT_STATUS_CHANGE: u32 = 34;
    pub const BANDWIDTH_REQUEST: u32 = 35;
    pub const DOORBELL_EVENT: u32 = 36;
    pub const HOST_CONTROLLER_EVENT: u32 = 37;
    pub const DEVICE_NOTIFICATION: u32 = 38;
    pub const MFINDEX_WRAP: u32 = 39;
}

/// Completion Codes
pub mod completion_code {
    pub const INVALID: u8 = 0;
    pub const SUCCESS: u8 = 1;
    pub const DATA_BUFFER_ERROR: u8 = 2;
    pub const BABBLE_DETECTED: u8 = 3;
    pub const USB_TRANSACTION_ERROR: u8 = 4;
    pub const TRB_ERROR: u8 = 5;
    pub const STALL_ERROR: u8 = 6;
    pub const RESOURCE_ERROR: u8 = 7;
    pub const BANDWIDTH_ERROR: u8 = 8;
    pub const NO_SLOTS_AVAILABLE: u8 = 9;
    pub const INVALID_STREAM_TYPE: u8 = 10;
    pub const SLOT_NOT_ENABLED: u8 = 11;
    pub const ENDPOINT_NOT_ENABLED: u8 = 12;
    pub const SHORT_PACKET: u8 = 13;
    pub const RING_UNDERRUN: u8 = 14;
    pub const RING_OVERRUN: u8 = 15;
    pub const VF_EVENT_RING_FULL: u8 = 16;
    pub const PARAMETER_ERROR: u8 = 17;
    pub const BANDWIDTH_OVERRUN: u8 = 18;
    pub const CONTEXT_STATE_ERROR: u8 = 19;
    pub const NO_PING_RESPONSE: u8 = 20;
    pub const EVENT_RING_FULL: u8 = 21;
    pub const INCOMPATIBLE_DEVICE: u8 = 22;
    pub const MISSED_SERVICE_ERROR: u8 = 23;
    pub const COMMAND_RING_STOPPED: u8 = 24;
    pub const COMMAND_ABORTED: u8 = 25;
    pub const STOPPED: u8 = 26;
    pub const STOPPED_LENGTH_INVALID: u8 = 27;
    pub const STOPPED_SHORT_PACKET: u8 = 28;
    pub const MAX_EXIT_LATENCY_TOO_LARGE: u8 = 29;
    pub const ISOCH_BUFFER_OVERRUN: u8 = 31;
    pub const EVENT_LOST: u8 = 32;
    pub const UNDEFINED_ERROR: u8 = 33;
    pub const INVALID_STREAM_ID: u8 = 34;
    pub const SECONDARY_BANDWIDTH_ERROR: u8 = 35;
    pub const SPLIT_TRANSACTION_ERROR: u8 = 36;
}

// =============================================================================
// EXTENDED CAPABILITIES
// =============================================================================

/// Extended Capability IDs
pub mod ext_cap {
    pub const USB_LEGACY_SUPPORT: u8 = 1;
    pub const SUPPORTED_PROTOCOL: u8 = 2;
    pub const EXTENDED_POWER_MANAGEMENT: u8 = 3;
    pub const IO_VIRTUALIZATION: u8 = 4;
    pub const MESSAGE_INTERRUPT: u8 = 5;
    pub const LOCAL_MEMORY: u8 = 6;
    pub const USB_DEBUG: u8 = 10;
    pub const EXTENDED_MESSAGE_INTERRUPT: u8 = 17;
}

/// USB Legacy Support Capability (USBLEGSUP) - Extended Capability ID 1
pub mod usblegsup {
    /// BIOS owns the HC (bit 16) - BIOS deve limpar quando ceder controle
    pub const HC_BIOS_OWNED: u32 = 1 << 16;
    /// OS owns the HC (bit 24) - OS seta para pedir controle
    pub const HC_OS_OWNED: u32 = 1 << 24;
}
