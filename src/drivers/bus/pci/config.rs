//! # Layout do Espaço de Configuração PCI
//!
//! Este arquivo define as constantes para os offsets dos registradores
//! no espaço de configuração PCI. Cada dispositivo PCI tem pelo menos
//! 256 bytes de configuração (PCI 2.x) ou 4KB (PCIe).
//!
//! ## Header Type 0 (Dispositivo Normal):
//! Usado pela maioria dos dispositivos (GPUs, NICs, etc).
//!
//! ## Header Type 1 (PCI Bridge):
//! Usado por bridges PCI-to-PCI.
//!
//! ## Header Type 2 (CardBus Bridge):
//! Usado por bridges CardBus (obsoleto).

// =============================================================================
// HEADER COMUM (Bytes 0x00-0x0F)
// =============================================================================

/// Vendor ID (16 bits) - Identificador do fabricante.
/// 0xFFFF indica slot vazio.
pub const PCI_VENDOR_ID: u8 = 0x00;

/// Device ID (16 bits) - Identificador do produto.
pub const PCI_DEVICE_ID: u8 = 0x02;

/// Command Register (16 bits) - Controle do dispositivo.
/// Bits: I/O Enable, Memory Enable, Bus Master, etc.
pub const PCI_COMMAND: u8 = 0x04;

/// Status Register (16 bits) - Estado do dispositivo.
/// Bits: Capabilities, Master Abort, etc.
pub const PCI_STATUS: u8 = 0x06;

/// Revision ID (8 bits) - Revisão do hardware.
pub const PCI_REVISION: u8 = 0x08;

/// Programming Interface (8 bits) - Interface de programação.
pub const PCI_PROG_IF: u8 = 0x09;

/// Subclass Code (8 bits) - Subclasse do dispositivo.
pub const PCI_SUBCLASS: u8 = 0x0A;

/// Class Code (8 bits) - Classe principal do dispositivo.
pub const PCI_CLASS: u8 = 0x0B;

/// Cache Line Size (8 bits) - Tamanho da linha de cache.
pub const PCI_CACHE_LINE_SIZE: u8 = 0x0C;

/// Latency Timer (8 bits) - Timer de latência.
pub const PCI_LATENCY_TIMER: u8 = 0x0D;

/// Header Type (8 bits) - Tipo de header.
/// Bit 7: Multi-função.
/// Bits 6-0: Tipo (0, 1, ou 2).
pub const PCI_HEADER_TYPE: u8 = 0x0E;

/// BIST (8 bits) - Built-In Self Test.
pub const PCI_BIST: u8 = 0x0F;

// =============================================================================
// HEADER TYPE 0 (Bytes 0x10-0x3F) - DISPOSITIVO NORMAL
// =============================================================================

/// Base Address Register 0 (32 bits).
pub const PCI_BAR0: u8 = 0x10;

/// Base Address Register 1 (32 bits).
pub const PCI_BAR1: u8 = 0x14;

/// Base Address Register 2 (32 bits).
pub const PCI_BAR2: u8 = 0x18;

/// Base Address Register 3 (32 bits).
pub const PCI_BAR3: u8 = 0x1C;

/// Base Address Register 4 (32 bits).
pub const PCI_BAR4: u8 = 0x20;

/// Base Address Register 5 (32 bits).
pub const PCI_BAR5: u8 = 0x24;

/// CardBus CIS Pointer (32 bits).
pub const PCI_CARDBUS_CIS: u8 = 0x28;

/// Subsystem Vendor ID (16 bits).
pub const PCI_SUBSYSTEM_VENDOR_ID: u8 = 0x2C;

/// Subsystem ID (16 bits).
pub const PCI_SUBSYSTEM_ID: u8 = 0x2E;

/// Expansion ROM Base Address (32 bits).
pub const PCI_ROM_ADDRESS: u8 = 0x30;

/// Capabilities Pointer (8 bits) - Offset da primeira capability.
pub const PCI_CAPABILITIES_PTR: u8 = 0x34;

/// Interrupt Line (8 bits) - IRQ atribuída pelo BIOS.
pub const PCI_INTERRUPT_LINE: u8 = 0x3C;

/// Interrupt Pin (8 bits) - Pino de interrupção (INTA-INTD).
/// 0 = não usa, 1 = INTA, 2 = INTB, etc.
pub const PCI_INTERRUPT_PIN: u8 = 0x3D;

/// Min Grant (8 bits) - Tempo mínimo de bus grant.
pub const PCI_MIN_GNT: u8 = 0x3E;

/// Max Latency (8 bits) - Latência máxima aceita.
pub const PCI_MAX_LAT: u8 = 0x3F;

// =============================================================================
// HEADER TYPE 1 (PCI-TO-PCI BRIDGE)
// =============================================================================

/// Primary Bus Number (8 bits).
pub const PCI_PRIMARY_BUS: u8 = 0x18;

/// Secondary Bus Number (8 bits).
pub const PCI_SECONDARY_BUS: u8 = 0x19;

/// Subordinate Bus Number (8 bits).
pub const PCI_SUBORDINATE_BUS: u8 = 0x1A;

/// Secondary Latency Timer (8 bits).
pub const PCI_SEC_LATENCY_TIMER: u8 = 0x1B;

// =============================================================================
// BITS DO COMMAND REGISTER
// =============================================================================

/// Habilita resposta a I/O space.
pub const PCI_CMD_IO_SPACE: u16 = 0x0001;

/// Habilita resposta a Memory space.
pub const PCI_CMD_MEMORY_SPACE: u16 = 0x0002;

/// Habilita Bus Mastering (DMA).
pub const PCI_CMD_BUS_MASTER: u16 = 0x0004;

/// Habilita Special Cycles.
pub const PCI_CMD_SPECIAL_CYCLES: u16 = 0x0008;

/// Habilita Memory Write and Invalidate.
pub const PCI_CMD_MEM_WR_INV: u16 = 0x0010;

/// Habilita VGA Palette Snoop.
pub const PCI_CMD_VGA_PALETTE: u16 = 0x0020;

/// Habilita resposta de paridade.
pub const PCI_CMD_PARITY: u16 = 0x0040;

/// Habilita SERR# driver.
pub const PCI_CMD_SERR: u16 = 0x0100;

/// Habilita Fast Back-to-Back.
pub const PCI_CMD_FAST_B2B: u16 = 0x0200;

/// Desabilita interrupções INTx.
pub const PCI_CMD_INTERRUPT_DISABLE: u16 = 0x0400;

// =============================================================================
// CLASSES PCI
// =============================================================================

/// Dispositivo antigo (antes de PCI 2.0).
pub const PCI_CLASS_UNCLASSIFIED: u8 = 0x00;

/// Mass Storage Controller.
pub const PCI_CLASS_STORAGE: u8 = 0x01;

/// Network Controller.
pub const PCI_CLASS_NETWORK: u8 = 0x02;

/// Display Controller.
pub const PCI_CLASS_DISPLAY: u8 = 0x03;

/// Multimedia Controller.
pub const PCI_CLASS_MULTIMEDIA: u8 = 0x04;

/// Memory Controller.
pub const PCI_CLASS_MEMORY: u8 = 0x05;

/// Bridge Device.
pub const PCI_CLASS_BRIDGE: u8 = 0x06;

/// Simple Communication Controller.
pub const PCI_CLASS_COMMUNICATION: u8 = 0x07;

/// Base System Peripheral.
pub const PCI_CLASS_SYSTEM: u8 = 0x08;

/// Input Device Controller.
pub const PCI_CLASS_INPUT: u8 = 0x09;

/// Docking Station.
pub const PCI_CLASS_DOCKING: u8 = 0x0A;

/// Processor.
pub const PCI_CLASS_PROCESSOR: u8 = 0x0B;

/// Serial Bus Controller.
pub const PCI_CLASS_SERIAL_BUS: u8 = 0x0C;

/// Wireless Controller.
pub const PCI_CLASS_WIRELESS: u8 = 0x0D;

/// Intelligent Controller.
pub const PCI_CLASS_INTELLIGENT: u8 = 0x0E;

/// Satellite Communication.
pub const PCI_CLASS_SATELLITE: u8 = 0x0F;

/// Encryption Controller.
pub const PCI_CLASS_ENCRYPTION: u8 = 0x10;

/// Signal Processing Controller.
pub const PCI_CLASS_SIGNAL: u8 = 0x11;

// =============================================================================
// SUBCLASSES COMUNS
// =============================================================================

// Storage (0x01)
pub const PCI_SUBCLASS_SCSI: u8 = 0x00;
pub const PCI_SUBCLASS_IDE: u8 = 0x01;
pub const PCI_SUBCLASS_FLOPPY: u8 = 0x02;
pub const PCI_SUBCLASS_IPI: u8 = 0x03;
pub const PCI_SUBCLASS_RAID: u8 = 0x04;
pub const PCI_SUBCLASS_ATA: u8 = 0x05;
pub const PCI_SUBCLASS_SATA: u8 = 0x06;
pub const PCI_SUBCLASS_SAS: u8 = 0x07;
pub const PCI_SUBCLASS_NVME: u8 = 0x08;

// Display (0x03)
pub const PCI_SUBCLASS_VGA: u8 = 0x00;
pub const PCI_SUBCLASS_XGA: u8 = 0x01;
pub const PCI_SUBCLASS_3D: u8 = 0x02;

// Bridge (0x06)
pub const PCI_SUBCLASS_HOST_BRIDGE: u8 = 0x00;
pub const PCI_SUBCLASS_ISA_BRIDGE: u8 = 0x01;
pub const PCI_SUBCLASS_EISA_BRIDGE: u8 = 0x02;
pub const PCI_SUBCLASS_MCA_BRIDGE: u8 = 0x03;
pub const PCI_SUBCLASS_PCI_BRIDGE: u8 = 0x04;
pub const PCI_SUBCLASS_PCMCIA_BRIDGE: u8 = 0x05;
pub const PCI_SUBCLASS_NUBUS_BRIDGE: u8 = 0x06;
pub const PCI_SUBCLASS_CARDBUS_BRIDGE: u8 = 0x07;
pub const PCI_SUBCLASS_RACEWAY_BRIDGE: u8 = 0x08;
pub const PCI_SUBCLASS_OTHER_BRIDGE: u8 = 0x80;

// Serial Bus (0x0C)
pub const PCI_SUBCLASS_FIREWIRE: u8 = 0x00;
pub const PCI_SUBCLASS_ACCESS_BUS: u8 = 0x01;
pub const PCI_SUBCLASS_SSA: u8 = 0x02;
pub const PCI_SUBCLASS_USB: u8 = 0x03;
pub const PCI_SUBCLASS_FIBRE: u8 = 0x04;
pub const PCI_SUBCLASS_SMBUS: u8 = 0x05;

// USB Prog IF (quando class=0x0C, subclass=0x03)
pub const PCI_PROG_IF_UHCI: u8 = 0x00;
pub const PCI_PROG_IF_OHCI: u8 = 0x10;
pub const PCI_PROG_IF_EHCI: u8 = 0x20;
pub const PCI_PROG_IF_XHCI: u8 = 0x30;
