//! Mapper: Funções de mapeamento de memória
use super::vmm::MapFlags;

/// Traduz endereço virtual para físico
pub fn translate_addr(addr: u64) -> Option<u64> {
    // Stub
    Some(addr) // Identity map fake
}

/// Mapeia uma página
pub fn map_page(page: u64, frame: u64, flags: MapFlags) -> Result<(), &'static str> {
    // Stub
    Ok(())
}

/// Desmapeia uma página
pub fn unmap_page(page: u64) -> Result<(), &'static str> {
    // Stub
    Ok(())
}

/// Mapeia página usando um alocador específico (se necessário)
pub fn map_page_with_pmm(
    page: u64,
    frame: u64,
    flags: MapFlags,
    _pmm: &mut crate::mm::pmm::BitmapFrameAllocator,
) -> Result<(), &'static str> {
    // Stub
    map_page(page, frame, flags)
}
