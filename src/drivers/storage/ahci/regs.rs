//! # Registradores e Constantes AHCI
//!
//! Definições de registradores MMIO e constantes do AHCI (Advanced Host Controller Interface).

/// Vendor ID da Intel (muitos controladores AHCI são Intel)
pub const INTEL_VENDOR_ID: u16 = 0x8086;

/// PCI Class Code para Mass Storage Controller
pub const PCI_CLASS_STORAGE: u8 = 0x01;

/// PCI Subclass para SATA Controller
pub const PCI_SUBCLASS_SATA: u8 = 0x06;

/// PCI Programming Interface para AHCI
pub const PCI_PROG_IF_AHCI: u8 = 0x01;

/// Número máximo de portas AHCI
pub const MAX_PORTS: usize = 32;

/// Signature de dispositivos SATA
pub mod signatures {
    /// ATA device
    pub const SATA_ATA: u32 = 0x0000_0101;
    /// ATAPI device
    pub const SATA_ATAPI: u32 = 0xEB14_0101;
    /// Enclosure management bridge
    pub const SATA_SEMB: u32 = 0xC33C_0101;
    /// Port multiplier
    pub const SATA_PM: u32 = 0x9669_0101;
}

/// Offsets dos registradores HBA (Host Bus Adapter) Generic
pub mod hba {
    /// Host Capabilities
    pub const CAP: u64 = 0x00;
    /// Global Host Control
    pub const GHC: u64 = 0x04;
    /// Interrupt Status
    pub const IS: u64 = 0x08;
    /// Ports Implemented
    pub const PI: u64 = 0x0C;
    /// Version
    pub const VS: u64 = 0x10;
    /// Command Completion Coalescing Control
    pub const CCC_CTL: u64 = 0x14;
    /// Command Completion Coalescing Ports
    pub const CCC_PORTS: u64 = 0x18;
    /// Enclosure Management Location
    pub const EM_LOC: u64 = 0x1C;
    /// Enclosure Management Control
    pub const EM_CTL: u64 = 0x20;
    /// Host Capabilities Extended
    pub const CAP2: u64 = 0x24;
    /// BIOS/OS Handoff Control and Status
    pub const BOHC: u64 = 0x28;
}

/// Bits do GHC (Global Host Control)
pub mod ghc {
    /// HBA Reset
    pub const HR: u32 = 1 << 0;
    /// Interrupt Enable
    pub const IE: u32 = 1 << 1;
    /// MSI Revert to Single Message
    pub const MRSM: u32 = 1 << 2;
    /// AHCI Enable
    pub const AE: u32 = 1 << 31;
}

/// Bits do CAP (Capabilities)
pub mod cap {
    /// Number of Ports (bits 4:0)
    pub const NP_MASK: u32 = 0x1F;
    /// Supports External SATA
    pub const SXS: u32 = 1 << 5;
    /// Enclosure Management Supported
    pub const EMS: u32 = 1 << 6;
    /// Command Completion Coalescing Supported
    pub const CCCS: u32 = 1 << 7;
    /// Number of Command Slots (bits 12:8)
    pub const NCS_SHIFT: u32 = 8;
    pub const NCS_MASK: u32 = 0x1F << NCS_SHIFT;
    /// Partial State Capable
    pub const PSC: u32 = 1 << 13;
    /// Slumber State Capable
    pub const SSC: u32 = 1 << 14;
    /// PIO Multiple DRQ Block
    pub const PMD: u32 = 1 << 15;
    /// FIS-based Switching Supported
    pub const FBSS: u32 = 1 << 16;
    /// Supports Port Multiplier
    pub const SPM: u32 = 1 << 17;
    /// Supports AHCI mode only
    pub const SAM: u32 = 1 << 18;
    /// Supports Native Command Queuing
    pub const SNCQ: u32 = 1 << 30;
    /// Supports 64-bit Addressing
    pub const S64A: u32 = 1 << 31;
}

/// Offset base para registradores de porta (porta n = 0x100 + n*0x80)
pub const PORT_BASE: u64 = 0x100;
/// Tamanho de cada bloco de registradores de porta
pub const PORT_SIZE: u64 = 0x80;

/// Offsets dos registradores de porta (relativos ao início da porta)
pub mod port {
    /// Port Command List Base Address (lower 32 bits)
    pub const CLB: u64 = 0x00;
    /// Port Command List Base Address (upper 32 bits)
    pub const CLBU: u64 = 0x04;
    /// Port FIS Base Address (lower 32 bits)
    pub const FB: u64 = 0x08;
    /// Port FIS Base Address (upper 32 bits)
    pub const FBU: u64 = 0x0C;
    /// Port Interrupt Status
    pub const IS: u64 = 0x10;
    /// Port Interrupt Enable
    pub const IE: u64 = 0x14;
    /// Port Command and Status
    pub const CMD: u64 = 0x18;
    /// Port Task File Data
    pub const TFD: u64 = 0x20;
    /// Port Signature
    pub const SIG: u64 = 0x24;
    /// Port Serial ATA Status (SCR0: SStatus)
    pub const SSTS: u64 = 0x28;
    /// Port Serial ATA Control (SCR2: SControl)
    pub const SCTL: u64 = 0x2C;
    /// Port Serial ATA Error (SCR1: SError)
    pub const SERR: u64 = 0x30;
    /// Port Serial ATA Active (SCR3: SActive)
    pub const SACT: u64 = 0x34;
    /// Port Command Issue
    pub const CI: u64 = 0x38;
    /// Port Serial ATA Notification
    pub const SNTF: u64 = 0x3C;
    /// Port FIS-based Switching Control
    pub const FBS: u64 = 0x40;
}

/// Bits do Port Command and Status (PxCMD)
pub mod port_cmd {
    /// Start (command processing)
    pub const ST: u32 = 1 << 0;
    /// Spin-Up Device
    pub const SUD: u32 = 1 << 1;
    /// Power On Device
    pub const POD: u32 = 1 << 2;
    /// Command List Override
    pub const CLO: u32 = 1 << 3;
    /// FIS Receive Enable
    pub const FRE: u32 = 1 << 4;
    /// Current Command Slot (bits 12:8)
    pub const CCS_SHIFT: u32 = 8;
    pub const CCS_MASK: u32 = 0x1F << CCS_SHIFT;
    /// Mechanical Presence Switch State
    pub const MPSS: u32 = 1 << 13;
    /// FIS Receive Running
    pub const FR: u32 = 1 << 14;
    /// Command List Running
    pub const CR: u32 = 1 << 15;
    /// Cold Presence State
    pub const CPS: u32 = 1 << 16;
    /// Port Multiplier Attached
    pub const PMA: u32 = 1 << 17;
    /// Hot Plug Capable Port
    pub const HPCP: u32 = 1 << 18;
    /// Mechanical Presence Switch Attached
    pub const MPSP: u32 = 1 << 19;
    /// Cold Presence Detection
    pub const CPD: u32 = 1 << 20;
    /// External SATA Port
    pub const ESP: u32 = 1 << 21;
    /// FIS-based Switching Capable Port
    pub const FBSCP: u32 = 1 << 22;
    /// Automatic Partial to Slumber Transitions Enabled
    pub const APSTE: u32 = 1 << 23;
    /// Device is ATAPI
    pub const ATAPI: u32 = 1 << 24;
    /// Drive LED on ATAPI Enable
    pub const DLAE: u32 = 1 << 25;
    /// Aggressive Link Power Management Enable
    pub const ALPE: u32 = 1 << 26;
    /// Aggressive Slumber/Partial
    pub const ASP: u32 = 1 << 27;
    /// Interface Communication Control (bits 31:28)
    pub const ICC_SHIFT: u32 = 28;
    pub const ICC_MASK: u32 = 0xF << ICC_SHIFT;
}

/// Bits do Port Serial ATA Status (PxSSTS)
pub mod port_ssts {
    /// Device Detection (bits 3:0)
    pub const DET_MASK: u32 = 0x0F;
    /// Interface Speed (bits 7:4)
    pub const SPD_SHIFT: u32 = 4;
    pub const SPD_MASK: u32 = 0x0F << SPD_SHIFT;
    /// Interface Power Management (bits 11:8)
    pub const IPM_SHIFT: u32 = 8;
    pub const IPM_MASK: u32 = 0x0F << IPM_SHIFT;
}

/// Valores de Device Detection (DET)
pub mod det {
    /// No device detected
    pub const NONE: u32 = 0;
    /// Device present but no communication
    pub const PRESENT: u32 = 1;
    /// Device present and communication established
    pub const COMM: u32 = 3;
    /// Phy in offline mode
    pub const OFFLINE: u32 = 4;
}

/// Valores de Interface Power Management (IPM)
pub mod ipm {
    /// Device not present or communication not established
    pub const NONE: u32 = 0;
    /// Active state
    pub const ACTIVE: u32 = 1;
    /// Partial power management state
    pub const PARTIAL: u32 = 2;
    /// Slumber power management state
    pub const SLUMBER: u32 = 6;
}
