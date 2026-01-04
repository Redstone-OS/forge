//! # xHCI Registers
//!
//! Definições dos registradores xHCI.
//!
//! ## Layout de Memória:
//! ```text
//! Base + 0x00: Capability Registers
//! Base + Cap.CAPLENGTH: Operational Registers
//! Base + Cap.RTSOFF: Runtime Registers
//! Base + Cap.DBOFF: Doorbell Registers
//! ```

// =============================================================================
// CAPABILITY REGISTERS
// =============================================================================

/// Capability Registers (leitura apenas)
#[repr(C)]
pub struct CapabilityRegs {
    /// CAPLENGTH (byte 0) + HCIVERSION (bytes 1-2)
    pub caplength_hciversion: u32,
    /// Structural Parameters 1
    pub hcsparams1: u32,
    /// Structural Parameters 2
    pub hcsparams2: u32,
    /// Structural Parameters 3
    pub hcsparams3: u32,
    /// Capability Parameters 1
    pub hccparams1: u32,
    /// Doorbell Offset
    pub dboff: u32,
    /// Runtime Register Space Offset
    pub rtsoff: u32,
    /// Capability Parameters 2
    pub hccparams2: u32,
}

impl CapabilityRegs {
    /// Retorna CAPLENGTH (offset para operational regs).
    pub fn caplength(&self) -> u8 {
        (self.caplength_hciversion & 0xFF) as u8
    }

    /// Retorna versão do HCI (BCD).
    pub fn hci_version(&self) -> u16 {
        ((self.caplength_hciversion >> 16) & 0xFFFF) as u16
    }

    /// Retorna número de slots suportados.
    pub fn max_slots(&self) -> u8 {
        (self.hcsparams1 & 0xFF) as u8
    }

    /// Retorna número de interrupters.
    pub fn max_interrupters(&self) -> u16 {
        ((self.hcsparams1 >> 8) & 0x7FF) as u16
    }

    /// Retorna número de portas.
    pub fn max_ports(&self) -> u8 {
        ((self.hcsparams1 >> 24) & 0xFF) as u8
    }

    /// Retorna se é 64-bit capable.
    pub fn is_64bit(&self) -> bool {
        (self.hccparams1 & 0x01) != 0
    }

    /// Retorna context size (32 ou 64 bytes).
    pub fn context_size(&self) -> usize {
        if (self.hccparams1 & 0x04) != 0 {
            64
        } else {
            32
        }
    }
}

// =============================================================================
// OPERATIONAL REGISTERS
// =============================================================================

/// Operational Registers
#[repr(C)]
pub struct OperationalRegs {
    /// USB Command
    pub usbcmd: u32,
    /// USB Status
    pub usbsts: u32,
    /// Page Size
    pub pagesize: u32,
    /// Reserved
    pub _reserved1: [u32; 2],
    /// Device Notification Control
    pub dnctrl: u32,
    /// Command Ring Control
    pub crcr: u64,
    /// Reserved
    pub _reserved2: [u32; 4],
    /// Device Context Base Address Array Pointer
    pub dcbaap: u64,
    /// Configure
    pub config: u32,
}

// USBCMD bits
pub const USBCMD_RUN: u32 = 1 << 0;
pub const USBCMD_HCRST: u32 = 1 << 1;
pub const USBCMD_INTE: u32 = 1 << 2;
pub const USBCMD_HSEE: u32 = 1 << 3;
pub const USBCMD_LHCRST: u32 = 1 << 7;
pub const USBCMD_CSS: u32 = 1 << 8;
pub const USBCMD_CRS: u32 = 1 << 9;
pub const USBCMD_EWE: u32 = 1 << 10;
pub const USBCMD_EU3S: u32 = 1 << 11;

// USBSTS bits
pub const USBSTS_HCH: u32 = 1 << 0; // Controller Halted
pub const USBSTS_HSE: u32 = 1 << 2; // Host System Error
pub const USBSTS_EINT: u32 = 1 << 3; // Event Interrupt
pub const USBSTS_PCD: u32 = 1 << 4; // Port Change Detect
pub const USBSTS_SSS: u32 = 1 << 8; // Save State Status
pub const USBSTS_RSS: u32 = 1 << 9; // Restore State Status
pub const USBSTS_SRE: u32 = 1 << 10; // Save/Restore Error
pub const USBSTS_CNR: u32 = 1 << 11; // Controller Not Ready
pub const USBSTS_HCE: u32 = 1 << 12; // Host Controller Error

// CRCR bits
pub const CRCR_RCS: u64 = 1 << 0; // Ring Cycle State
pub const CRCR_CS: u64 = 1 << 1; // Command Stop
pub const CRCR_CA: u64 = 1 << 2; // Command Abort
pub const CRCR_CRR: u64 = 1 << 3; // Command Ring Running

// =============================================================================
// PORT REGISTERS
// =============================================================================

/// Port Register Set (um por porta)
#[repr(C)]
pub struct PortRegs {
    /// Port Status and Control
    pub portsc: u32,
    /// Port PM Status and Control
    pub portpmsc: u32,
    /// Port Link Info
    pub portli: u32,
    /// Port Hardware LPM Control
    pub porthlpmc: u32,
}

// PORTSC bits
pub const PORTSC_CCS: u32 = 1 << 0; // Current Connect Status
pub const PORTSC_PED: u32 = 1 << 1; // Port Enabled/Disabled
pub const PORTSC_OCA: u32 = 1 << 3; // Overcurrent Active
pub const PORTSC_PR: u32 = 1 << 4; // Port Reset
pub const PORTSC_PP: u32 = 1 << 9; // Port Power
pub const PORTSC_CSC: u32 = 1 << 17; // Connect Status Change
pub const PORTSC_PEC: u32 = 1 << 18; // Port Enable/Disable Change
pub const PORTSC_WRC: u32 = 1 << 19; // Warm Port Reset Change
pub const PORTSC_OCC: u32 = 1 << 20; // Overcurrent Change
pub const PORTSC_PRC: u32 = 1 << 21; // Port Reset Change
pub const PORTSC_PLC: u32 = 1 << 22; // Port Link State Change
pub const PORTSC_CEC: u32 = 1 << 23; // Port Config Error Change
pub const PORTSC_WCE: u32 = 1 << 25; // Wake on Connect Enable
pub const PORTSC_WDE: u32 = 1 << 26; // Wake on Disconnect Enable
pub const PORTSC_WOE: u32 = 1 << 27; // Wake on Overcurrent Enable
pub const PORTSC_DR: u32 = 1 << 30; // Device Removable
pub const PORTSC_WPR: u32 = 1 << 31; // Warm Port Reset

// PORTSC masks
pub const PORTSC_PLS_MASK: u32 = 0xF << 5; // Port Link State
pub const PORTSC_SPEED_MASK: u32 = 0xF << 10; // Port Speed

// =============================================================================
// RUNTIME REGISTERS
// =============================================================================

/// Runtime Registers
#[repr(C)]
pub struct RuntimeRegs {
    /// Microframe Index
    pub mfindex: u32,
    /// Reserved
    pub _reserved: [u32; 7],
    // Followed by Interrupter Register Sets (one per interrupter)
}

/// Interrupter Register Set
#[repr(C)]
pub struct InterrupterRegs {
    /// Interrupter Management
    pub iman: u32,
    /// Interrupter Moderation
    pub imod: u32,
    /// Event Ring Segment Table Size
    pub erstsz: u32,
    /// Reserved
    pub _reserved: u32,
    /// Event Ring Segment Table Base Address
    pub erstba: u64,
    /// Event Ring Dequeue Pointer
    pub erdp: u64,
}

// IMAN bits
pub const IMAN_IP: u32 = 1 << 0; // Interrupt Pending
pub const IMAN_IE: u32 = 1 << 1; // Interrupt Enable
