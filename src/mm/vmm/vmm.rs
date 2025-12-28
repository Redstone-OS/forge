use bitflags::bitflags;
// TODO NAO USAR DEPENDENCIA EXTERNA
bitflags! {
    /// Flags de mapeamento de página (Paging Flags)
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MapFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
        const NO_EXECUTE = 1 << 63;
    }
}

/// Estrutura representando uma Page Table nível 4 (PML4)
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u64; 512],
}

/// Inicializa o subsistema VMM
pub fn init(_boot_info: &crate::core::boot::handoff::BootInfo) {
    crate::kinfo!("(VMM) Inicializando VMM...");
    // TODO: Implementar init real (trocar para page table do kernel, etc)
}
