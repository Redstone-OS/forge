use crate::mm::addr::PhysAddr;

/// Tipos de região de memória (baseado no UEFI/Multiboot)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    Kernel,
    KernelStack,
    PageTable,
    Bootloader,
    FrameZero, // Primeiros 4k ou região legacy
    Unknown,
}

/// Uma região contígua de memória física
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: PhysAddr,
    pub end: PhysAddr,
    pub kind: MemoryRegionType,
}

impl MemoryRegion {
    /// Número de frames nessa região
    pub fn frame_count(&self) -> usize {
        let size = self.end.as_u64() - self.start.as_u64();
        (size / crate::mm::config::PAGE_SIZE as u64) as usize
    }
}
