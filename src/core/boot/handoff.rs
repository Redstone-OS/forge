/// Arquivo: core/boot/handoff.rs
///
/// Propósito: Definição das estruturas de dados passadas do Bootloader para o Kernel.
/// Contém o mapa de memória, informações de vídeo (framebuffer), tabelas ACPI, etc.
///
/// Detalhes de Implementação:
/// - Estruturas `repr(C)` para garantir layout binário compatível.
/// - Deve coincidir exatamente com o que o bootloader (Ignite) preenche.

// Handoff Data (Bootloader -> Kernel)

/// Assinatura mágica para validar que o BootInfo é legítimo ("REDSTONE" em ASCII).
pub const BOOT_INFO_MAGIC: u64 = 0x524544_53544F4E45;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    /// Assinatura mágica (deve ser verificada pelo Kernel).
    pub magic: u64,

    /// Versão do protocolo de boot.
    pub version: u32,

    /// Padding para alinhamento de 8 bytes (campos seguintes são u64).
    /// O kernel DEVE ter este campo também para manter ABI.
    pub _padding: u32,

    /// Informações de vídeo (GOP).
    pub framebuffer: FramebufferInfo,

    /// Mapa de memória física (ponteiro e tamanho).
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

    /// Endereço FÍSICO do CR3 (PML4) configurado pelo bootloader.
    /// O kernel herda esta hierarquia de page tables.
    pub cr3_phys: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    /// Endereço físico do buffer de pixels.
    pub addr: u64,
    /// Tamanho total em bytes.
    pub size: u64,
    /// Largura em pixels.
    pub width: u32,
    /// Altura em pixels.
    pub height: u32,
    /// Pixels por linha (stride).
    pub stride: u32,
    /// Formato de pixel.
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

/// Entrada do mapa de memória física
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
