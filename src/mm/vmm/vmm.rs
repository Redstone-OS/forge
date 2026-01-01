use crate::bitflags;
// TODO NAO USAR DEPENDENCIA EXTERNA
bitflags! {
    /// Flags de mapeamento de página (Paging Flags)
    pub struct MapFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
        const EXECUTABLE = 1 << 9; // Flag de controle interna
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
    // Captura o CR3 do Bootloader (Kernel P4 com Identity Map)
    // Isso é crucial para que possamos temporariamente trocar para este CR3
    // quando precisarmos modificar Page Tables de processos que não têm Identity Map.
    let cr3 = super::mapper::read_cr3();
    KERNEL_CR3.store(cr3, core::sync::atomic::Ordering::SeqCst);
    crate::kinfo!("(VMM) Kernel CR3 saved: ", cr3);
}

/// CR3 do Kernel (Bootloader)
pub static KERNEL_CR3: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
