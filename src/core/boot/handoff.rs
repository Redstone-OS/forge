//! Interface de Handoff (Bootloader -> Kernel).
//! Define a estrutura de dados (ABI) passada do Ignite para o Forge.

/// Assinatura mágica esperada do Bootloader ("REDSTONE").
pub const BOOT_MAGIC: u64 = 0x524544_53544F4E45;

/// Versão do protocolo de boot.
pub const BOOT_INFO_VERSION: u32 = 2;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    pub magic: u64,
    pub version: u32,
    pub _padding: u32,
    pub framebuffer: FramebufferInfo,
    pub memory_map_addr: u64,
    pub memory_map_len: u64,
    pub rsdp_addr: u64,
    pub kernel_phys_addr: u64,
    pub kernel_size: u64,
    pub initramfs_addr: u64,
    pub initramfs_size: u64,
    pub cr3_phys: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub addr: u64,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr = 1,
    Bitmask = 2,
    BltOnly = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    pub base: u64,
    pub len: u64,
    pub typ: MemoryType,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    Usable = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    AcpiNvs = 4,
    BadMemory = 5,
    BootloaderReclaimable = 6,
    KernelAndModules = 7,
    Framebuffer = 8,
}
