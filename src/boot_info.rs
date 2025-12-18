//! Boot Information Structure
//!
//! Estrutura compartilhada entre bootloader e kernel.
//! Contém informações sobre hardware e recursos disponíveis.

#![no_std]

/// Informações de boot passadas do bootloader para o kernel
///
/// Esta estrutura é escrita pelo bootloader em um endereço fixo (0x8000)
/// e lida pelo kernel durante a inicialização.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    /// Endereço físico do framebuffer
    pub fb_addr: u64,

    /// Largura do framebuffer em pixels
    pub fb_width: u32,

    /// Altura do framebuffer em pixels
    pub fb_height: u32,

    /// Stride (pixels por linha, pode ser > width devido a padding)
    pub fb_stride: u32,

    /// Formato de pixel (0=RGB, 1=BGR)
    pub fb_format: u32,

    /// Endereço base do kernel na memória
    pub kernel_base: u64,

    /// Tamanho do kernel em bytes
    pub kernel_size: u64,

    /// Endereço do InitFS (0 se não presente)
    pub initfs_addr: u64,

    /// Tamanho do InitFS em bytes (0 se não presente)
    pub initfs_size: u64,

    /// Endereço do memory map (0 se não disponível)
    pub memory_map_addr: u64,

    /// Tamanho do memory map em bytes (0 se não disponível)
    pub memory_map_size: u64,
}

/// Tipo de região de memória
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    /// Memória utilizável
    Usable = 1,

    /// Reservada (BIOS, ACPI, etc)
    Reserved = 2,

    /// ACPI Reclaimable
    AcpiReclaimable = 3,

    /// ACPI NVS
    AcpiNvs = 4,

    /// Bad memory
    BadMemory = 5,

    /// Kernel/Bootloader
    KernelReserved = 6,
}

/// Entrada do memory map
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    /// Endereço físico inicial
    pub base: u64,

    /// Tamanho em bytes
    pub length: u64,

    /// Tipo de região
    pub region_type: MemoryRegionType,

    /// Padding para alinhamento
    _padding: u32,
}

impl MemoryRegion {
    /// Cria nova região de memória
    pub const fn new(base: u64, length: u64, region_type: MemoryRegionType) -> Self {
        Self {
            base,
            length,
            region_type,
            _padding: 0,
        }
    }
}

impl BootInfo {
    /// Endereço fixo onde o BootInfo é armazenado
    pub const ADDRESS: usize = 0x8000;

    /// Cria um novo BootInfo vazio
    pub const fn new() -> Self {
        Self {
            fb_addr:         0,
            fb_width:        0,
            fb_height:       0,
            fb_stride:       0,
            fb_format:       0,
            kernel_base:     0,
            kernel_size:     0,
            initfs_addr:     0,
            initfs_size:     0,
            memory_map_addr: 0,
            memory_map_size: 0,
        }
    }

    /// Lê o BootInfo do endereço fixo (usado pelo kernel)
    ///
    /// # Safety
    /// Assume que o bootloader já escreveu um BootInfo válido em 0x8000
    pub unsafe fn read() -> &'static Self {
        unsafe { &*(Self::ADDRESS as *const BootInfo) }
    }

    /// Escreve o BootInfo no endereço fixo (usado pelo bootloader)
    ///
    /// # Safety
    /// Escreve diretamente na memória física
    pub unsafe fn write(&self) {
        unsafe {
            let ptr = Self::ADDRESS as *mut BootInfo;
            ptr.write(*self);
        }
    }
}
