//! Interface de Handoff (Bootloader -> Kernel).
//! Define a estrutura de dados (ABI) passada do Ignite para o Forge.
//!
//! # Industrial Standard
//! - Structs `#[repr(C)]` para garantia de layout.
//! - Tipos primitivos (`u64`, `u32`) para portabilidade.
//! - Magic Number para validação de versão.

/// Assinatura mágica esperada do Bootloader ("REDSTONE").
/// Usado para garantir que não estamos bootando lixo.
pub const BOOT_MAGIC: u64 = 0x524544_53544F4E45;

/// Estrutura de informações de boot.
/// Deve ser mantida em sincronia binária exata com o Bootloader.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    /// Assinatura para validação (deve ser igual a BOOT_MAGIC).
    pub magic: u64,

    /// Versão do protocolo de boot.
    pub version: u32,

    /// Informações de vídeo (GOP).
    pub framebuffer: FramebufferInfo,

    /// Mapa de memória física.
    pub memory_map_addr: u64,
    pub memory_map_len: u64,

    /// Tabela ACPI RSDP (Root System Description Pointer).
    pub rsdp_addr: u64,

    /// Localização física do Kernel.
    pub kernel_phys_addr: u64,
    pub kernel_size: u64,

    /// Endereço do Initramfs (se carregado).
    pub initramfs_addr: u64,
    pub initramfs_size: u64,
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
