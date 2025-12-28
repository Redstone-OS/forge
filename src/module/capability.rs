//! Capabilities específicas de módulos

/// Tipos de capability que módulos podem requisitar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum ModuleCapType {
    /// Acesso a DMA (requer IOMMU)
    DmaAccess = 1 << 0,
    /// Registrar handler de IRQ
    IrqHandler = 1 << 1,
    /// Acessar região MMIO específica
    MmioAccess = 1 << 2,
    /// Acessar portas IO
    IoPortAccess = 1 << 3,
    /// Alocar memória física contígua
    PhysAlloc = 1 << 4,
    /// Registrar block device
    BlockDevice = 1 << 5,
    /// Registrar char device
    CharDevice = 1 << 6,
    /// Registrar network device
    NetDevice = 1 << 7,
    /// Acessar config PCI
    PciConfig = 1 << 8,
}

/// Capability concedida a módulo
pub struct ModuleCapability {
    pub cap_type: ModuleCapType,
    /// Parâmetros específicos (ex: range de MMIO)
    pub param0: u64,
    pub param1: u64,
}

impl ModuleCapability {
    pub const fn new(cap_type: ModuleCapType) -> Self {
        Self {
            cap_type,
            param0: 0,
            param1: 0,
        }
    }
    
    pub const fn with_range(cap_type: ModuleCapType, start: u64, end: u64) -> Self {
        Self {
            cap_type,
            param0: start,
            param1: end,
        }
    }
}
