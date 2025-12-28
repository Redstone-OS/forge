/// Arquivo: core/boot/handoff.rs
///
/// Propósito: Definição das estruturas de dados passadas do Bootloader para o Kernel.
/// Contém o mapa de memória, informações de vídeo (framebuffer), tabelas ACPI, etc.
///
/// Detalhes de Implementação:
/// - Estruturas `repr(C)` para garantir layout binário compatível.
/// - Deve coincidir exatamente com o que o bootloader (Ignite) preenche.

// Handoff Data (Bootloader -> Kernel)

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    /// Versão da estrutura de boot info (para compatibilidade)
    pub version: u64,

    /// O Mapa de Memória do sistema
    pub memory_map: MemoryMap,

    /// Informações do Framebuffer Gráfico (GOP/VESA)
    pub framebuffer: Framebuffer,

    /// Endereço físico da tabela rsdp (ACPI)
    /// Se 0, não encontrado.
    pub rsdp_addr: u64,

    /// Endereço físico inicial do Initramfs (se carregado)
    pub initramfs_addr: u64,
    /// Tamanho do Initramfs
    pub initramfs_size: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    pub regions: [MemoryRegion; 64], // Limite fixo simplificado para boot
    pub count: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    Usable = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    AcpiNvs = 4,
    BadMemory = 5,
    KernelCode = 6,
    KernelStack = 7,
    KernelData = 8,
    FrameBuffer = 9,
    Bootloader = 10,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
    pub kind: MemoryType,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Framebuffer {
    pub address: u64,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
}
