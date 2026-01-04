//! # Estruturas de Memória AHCI
//!
//! Define as estruturas de memória usadas pelo AHCI para DMA.
//!
//! ## Referências
//!
//! - AHCI Spec 1.3.1, Seção 4 (HBA Memory Registers)

use crate::mm::pmm::{FrameAllocator, FRAME_ALLOCATOR};
use crate::mm::{PhysAddr, VirtAddr};
use core::ptr;

/// Tamanho de um setor
pub const SECTOR_SIZE: usize = 512;

/// Número máximo de command slots (32)
pub const MAX_CMD_SLOTS: usize = 32;

/// Tamanho do Command List (32 headers * 32 bytes = 1KB, alinhado a 1KB)
pub const CMD_LIST_SIZE: usize = MAX_CMD_SLOTS * 32;

/// Tamanho do FIS Buffer (256 bytes, alinhado a 256)
pub const FIS_BUFFER_SIZE: usize = 256;

/// Tamanho de um Command Table (128 bytes header + PRDs)
pub const CMD_TABLE_SIZE: usize = 256; // 128 + espaço para alguns PRDs

// =============================================================================
// COMMAND HEADER (32 bytes cada, 32 no Command List)
// =============================================================================

/// Command Header structure (4 DWORDs = 32 bytes)
///
/// Cada Command List contém 32 destes headers.
#[repr(C, align(32))]
#[derive(Debug, Clone, Copy, Default)]
pub struct CommandHeader {
    /// DW0: Flags e tamanhos
    /// - Bits 4:0: Command FIS Length (DWORDS)
    /// - Bit 5: ATAPI
    /// - Bit 6: Write (1) / Read (0)
    /// - Bit 7: Prefetchable
    /// - Bits 15:8: Reset bit, BIST, Clear Busy, reserved
    /// - Bits 20:16: PMP (Port Multiplier Port)
    /// - Bits 31:16: PRDT Length (number of entries)
    pub dw0: u32,
    /// DW1: PRD Byte Count (bytes transferidos)
    pub prd_byte_count: u32,
    /// DW2: Command Table Base Address (lower 32-bit)
    pub ctba: u32,
    /// DW3: Command Table Base Address (upper 32-bit)
    pub ctbau: u32,
    /// DW4-7: Reserved
    pub reserved: [u32; 4],
}

impl CommandHeader {
    /// Cria um header para leitura
    pub fn for_read(ctb_phys: u64, prdt_count: u16) -> Self {
        Self {
            // FIS Length = 5 DWORDs (Register H2D FIS)
            // Write = 0 (leitura)
            // PRDT Length nos bits 16-31
            dw0: 5 | ((prdt_count as u32) << 16),
            prd_byte_count: 0,
            ctba: ctb_phys as u32,
            ctbau: (ctb_phys >> 32) as u32,
            reserved: [0; 4],
        }
    }
}

// =============================================================================
// FIS (Frame Information Structure)
// =============================================================================

/// Register Host to Device FIS (20 bytes)
///
/// Usado para enviar comandos ATA para o dispositivo.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FisRegH2D {
    /// FIS Type (0x27 para Register H2D)
    pub fis_type: u8,
    /// Bits 3:0: PM Port
    /// Bit 7: Command (1) / Control (0)
    pub pmport_and_c: u8,
    /// Command register
    pub command: u8,
    /// Feature register (low)
    pub feature_low: u8,

    /// LBA 0-7
    pub lba0: u8,
    /// LBA 8-15
    pub lba1: u8,
    /// LBA 16-23
    pub lba2: u8,
    /// Device register
    pub device: u8,

    /// LBA 24-31
    pub lba3: u8,
    /// LBA 32-39
    pub lba4: u8,
    /// LBA 40-47
    pub lba5: u8,
    /// Feature register (high)
    pub feature_high: u8,

    /// Sector count (low)
    pub count_low: u8,
    /// Sector count (high)
    pub count_high: u8,
    /// ISO command completion
    pub icc: u8,
    /// Control register
    pub control: u8,

    /// Reserved
    pub reserved: [u8; 4],
}

/// FIS Type constants
pub mod fis_type {
    pub const REG_H2D: u8 = 0x27; // Register FIS - Host to Device
    pub const REG_D2H: u8 = 0x34; // Register FIS - Device to Host
    pub const DMA_ACT: u8 = 0x39; // DMA Activate FIS
    pub const DMA_SETUP: u8 = 0x41; // DMA Setup FIS
    pub const DATA: u8 = 0x46; // Data FIS
    pub const BIST: u8 = 0x58; // BIST Activate FIS
    pub const PIO_SETUP: u8 = 0x5F; // PIO Setup FIS
    pub const DEV_BITS: u8 = 0xA1; // Set Device Bits FIS
}

/// ATA Commands
pub mod ata_cmd {
    pub const READ_DMA_EXT: u8 = 0x25; // Read DMA Extended (LBA48)
    pub const WRITE_DMA_EXT: u8 = 0x35; // Write DMA Extended (LBA48)
    pub const IDENTIFY: u8 = 0xEC; // Identify Device
}

impl FisRegH2D {
    /// Cria um FIS para READ DMA EXT
    pub fn read_dma_ext(lba: u64, sector_count: u16) -> Self {
        Self {
            fis_type: fis_type::REG_H2D,
            pmport_and_c: 0x80, // Command bit set
            command: ata_cmd::READ_DMA_EXT,
            feature_low: 0,

            lba0: (lba & 0xFF) as u8,
            lba1: ((lba >> 8) & 0xFF) as u8,
            lba2: ((lba >> 16) & 0xFF) as u8,
            device: 0x40, // LBA mode

            lba3: ((lba >> 24) & 0xFF) as u8,
            lba4: ((lba >> 32) & 0xFF) as u8,
            lba5: ((lba >> 40) & 0xFF) as u8,
            feature_high: 0,

            count_low: (sector_count & 0xFF) as u8,
            count_high: ((sector_count >> 8) & 0xFF) as u8,
            icc: 0,
            control: 0,

            reserved: [0; 4],
        }
    }

    /// Cria um FIS para IDENTIFY DEVICE
    pub fn identify() -> Self {
        Self {
            fis_type: fis_type::REG_H2D,
            pmport_and_c: 0x80, // Command bit set
            command: ata_cmd::IDENTIFY,
            device: 0,
            ..Default::default()
        }
    }
}

// =============================================================================
// PRDT (Physical Region Descriptor Table)
// =============================================================================

/// Physical Region Descriptor (16 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PrdtEntry {
    /// Data Base Address (lower 32-bit)
    pub dba: u32,
    /// Data Base Address (upper 32-bit)
    pub dbau: u32,
    /// Reserved
    pub reserved: u32,
    /// Byte Count (bit 0: Interrupt on Completion, bits 21:1: byte count - 1)
    pub dbc: u32,
}

impl PrdtEntry {
    /// Cria uma entrada PRDT
    pub fn new(phys_addr: u64, byte_count: u32, interrupt: bool) -> Self {
        let dbc = ((byte_count - 1) & 0x3FFFFF) | if interrupt { 1 << 31 } else { 0 };
        Self {
            dba: phys_addr as u32,
            dbau: (phys_addr >> 32) as u32,
            reserved: 0,
            dbc,
        }
    }
}

// =============================================================================
// COMMAND TABLE
// =============================================================================

/// Command Table structure
///
/// Layout:
/// - Offset 0x00: Command FIS (64 bytes)
/// - Offset 0x40: ATAPI Command (16 bytes)
/// - Offset 0x50: Reserved (48 bytes)
/// - Offset 0x80: PRDT entries (16 bytes each)
#[repr(C, align(128))]
#[derive(Debug)]
pub struct CommandTable {
    /// Command FIS
    pub cfis: [u8; 64],
    /// ATAPI Command
    pub acmd: [u8; 16],
    /// Reserved
    pub reserved: [u8; 48],
    /// PRDT (Physical Region Descriptor Table) - até 8 entradas para começar
    pub prdt: [PrdtEntry; 8],
}

impl Default for CommandTable {
    fn default() -> Self {
        Self {
            cfis: [0; 64],
            acmd: [0; 16],
            reserved: [0; 48],
            prdt: [PrdtEntry::default(); 8],
        }
    }
}

impl CommandTable {
    /// Define o Command FIS para leitura
    pub fn set_read_fis(&mut self, lba: u64, sector_count: u16) {
        let fis = FisRegH2D::read_dma_ext(lba, sector_count);
        // SAFETY: FisRegH2D é 20 bytes, cabe no cfis de 64 bytes
        unsafe {
            let fis_ptr = &fis as *const FisRegH2D as *const u8;
            ptr::copy_nonoverlapping(fis_ptr, self.cfis.as_mut_ptr(), 20);
        }
    }

    /// Define o Command FIS para IDENTIFY
    pub fn set_identify_fis(&mut self) {
        let fis = FisRegH2D::identify();
        unsafe {
            let fis_ptr = &fis as *const FisRegH2D as *const u8;
            ptr::copy_nonoverlapping(fis_ptr, self.cfis.as_mut_ptr(), 20);
        }
    }

    /// Configura uma entrada PRDT
    pub fn set_prdt(&mut self, index: usize, phys_addr: u64, byte_count: u32, interrupt: bool) {
        if index < self.prdt.len() {
            self.prdt[index] = PrdtEntry::new(phys_addr, byte_count, interrupt);
        }
    }
}

// =============================================================================
// RECEIVED FIS (256 bytes)
// =============================================================================

/// Received FIS structure (256 bytes, alinhado a 256)
#[repr(C, align(256))]
pub struct ReceivedFis {
    /// DMA Setup FIS
    pub dsfis: [u8; 28],
    pub reserved0: [u8; 4],
    /// PIO Setup FIS
    pub psfis: [u8; 20],
    pub reserved1: [u8; 12],
    /// D2H Register FIS
    pub rfis: [u8; 20],
    pub reserved2: [u8; 4],
    /// Set Device Bits FIS
    pub sdbfis: [u8; 8],
    /// Unknown FIS
    pub ufis: [u8; 64],
    /// Reserved
    pub reserved3: [u8; 96],
}

impl Default for ReceivedFis {
    fn default() -> Self {
        Self {
            dsfis: [0; 28],
            reserved0: [0; 4],
            psfis: [0; 20],
            reserved1: [0; 12],
            rfis: [0; 20],
            reserved2: [0; 4],
            sdbfis: [0; 8],
            ufis: [0; 64],
            reserved3: [0; 96],
        }
    }
}

// =============================================================================
// PORT MEMORY (estruturas por porta)
// =============================================================================

/// Estruturas de memória alocadas para uma porta AHCI
pub struct PortMemory {
    /// Command List (32 Command Headers)
    /// Físico: alinhado a 1024 bytes
    pub cmd_list_phys: PhysAddr,
    pub cmd_list_virt: VirtAddr,

    /// FIS Buffer (256 bytes)
    /// Físico: alinhado a 256 bytes
    pub fis_phys: PhysAddr,
    pub fis_virt: VirtAddr,

    /// Command Tables (uma para cada slot, 32 máximo)
    /// Físico: alinhado a 128 bytes
    pub cmd_tables_phys: [PhysAddr; MAX_CMD_SLOTS],
    pub cmd_tables_virt: [VirtAddr; MAX_CMD_SLOTS],
}

impl PortMemory {
    /// Aloca todas as estruturas de memória necessárias para uma porta
    ///
    /// ## Retorna
    ///
    /// `Some(PortMemory)` se alocação foi bem sucedida
    pub fn allocate() -> Option<Self> {
        // Alocar uma página para Command List (1KB) + FIS (256 bytes)
        // Ambos cabem em uma página de 4KB
        let frame = FRAME_ALLOCATOR.lock().allocate_frame()?;
        let page_virt = frame.as_u64(); // Em HHDM, phys == virt (ou usa conversão)

        // Command List está no início da página (alinhado a 1KB por default)
        let cmd_list_phys = frame;
        let cmd_list_virt = VirtAddr::new(page_virt);

        // FIS Buffer está após Command List (offset 1KB)
        let fis_phys = PhysAddr::new(frame.as_u64() + 1024);
        let fis_virt = VirtAddr::new(page_virt + 1024);

        // Alocar Command Tables (uma página para cada, por simplicidade)
        // Na prática, podemos otimizar para caber múltiplas em uma página
        let mut cmd_tables_phys = [PhysAddr::new(0); MAX_CMD_SLOTS];
        let mut cmd_tables_virt = [VirtAddr::new(0); MAX_CMD_SLOTS];

        // Alocar apenas 1 command table por enquanto (slot 0)
        if let Some(ct_frame) = FRAME_ALLOCATOR.lock().allocate_frame() {
            cmd_tables_phys[0] = ct_frame;
            cmd_tables_virt[0] = VirtAddr::new(ct_frame.as_u64());
        } else {
            // Falha na alocação, deveriamos liberar a primeira página
            // Por simplicidade, ignoramos o cleanup por agora
            return None;
        }

        // Zerar as estruturas
        unsafe {
            core::ptr::write_bytes(cmd_list_virt.as_u64() as *mut u8, 0, 4096);
            core::ptr::write_bytes(cmd_tables_virt[0].as_u64() as *mut u8, 0, 4096);
        }

        Some(Self {
            cmd_list_phys,
            cmd_list_virt,
            fis_phys,
            fis_virt,
            cmd_tables_phys,
            cmd_tables_virt,
        })
    }

    /// Obtém referência mutável para o Command Header no slot especificado
    pub fn cmd_header_mut(&self, slot: usize) -> &mut CommandHeader {
        assert!(slot < MAX_CMD_SLOTS);
        unsafe {
            let ptr = self.cmd_list_virt.as_u64() as *mut CommandHeader;
            &mut *ptr.add(slot)
        }
    }

    /// Obtém referência mutável para o Command Table no slot especificado
    pub fn cmd_table_mut(&self, slot: usize) -> &mut CommandTable {
        assert!(slot < MAX_CMD_SLOTS);
        assert!(
            self.cmd_tables_virt[slot].as_u64() != 0,
            "Command table not allocated"
        );
        unsafe {
            let ptr = self.cmd_tables_virt[slot].as_u64() as *mut CommandTable;
            &mut *ptr
        }
    }

    /// Obtém o endereço físico do Command Table no slot especificado
    pub fn cmd_table_phys(&self, slot: usize) -> PhysAddr {
        self.cmd_tables_phys[slot]
    }
}
