//! Estrutura de handoff do bootloader

/// Tipo de região de memória
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryRegionType {
    /// Memória utilizável
    Usable = 0,
    /// Reservada pelo firmware
    Reserved = 1,
    /// ACPI reclaimable
    AcpiReclaimable = 2,
    /// ACPI NVS
    AcpiNvs = 3,
    /// Região com defeito
    BadMemory = 4,
    /// Código do bootloader (pode ser reclamado)
    BootloaderReclaimable = 5,
    /// Código do kernel
    KernelAndModules = 6,
    /// Framebuffer
    Framebuffer = 7,
}

/// Uma região de memória física
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryRegion {
    pub base: u64,
    pub length: u64,
    pub region_type: MemoryRegionType,
}

/// Informações do framebuffer
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FramebufferInfo {
    pub address: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub bpp: u32,
}

/// Informações passadas pelo bootloader
/// 
/// # IMPORTANTE
/// 
/// Esta estrutura DEVE ser idêntica byte-a-byte à do bootloader!
/// Use apenas tipos primitivos e `#[repr(C)]`.
#[derive(Debug)]
#[repr(C)]
pub struct BootInfo {
    /// Mapa de memória física
    pub memory_map: &'static [MemoryRegion],
    
    /// Framebuffer (pode ser None)
    pub framebuffer: Option<FramebufferInfo>,
    
    /// Endereço físico das tabelas ACPI (RSDP)
    pub acpi_rsdp: Option<u64>,
    
    /// Linha de comando do kernel
    pub cmdline: Option<&'static str>,
    
    /// Endereço físico do initramfs
    pub initramfs_addr: Option<u64>,
    
    /// Tamanho do initramfs
    pub initramfs_size: u64,
}
