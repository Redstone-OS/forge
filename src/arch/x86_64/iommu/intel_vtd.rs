//! # Intel VT-d Driver
//!
//! Driver para Intel Virtualization Technology for Directed I/O.
//!
//! ## Detecção
//! VT-d é detectado via ACPI DMAR table.
//!
//! ## Nota
//! Esta é uma implementação stub. Implementação completa requer:
//! - Parsing de ACPI DMAR
//! - Configuração de DMA Remapping Hardware Units (DRHD)
//! - Gestão de page tables de IOMMU

use core::sync::atomic::{AtomicBool, Ordering};

/// Flag indicando se VT-d foi detectado
static VTD_AVAILABLE: AtomicBool = AtomicBool::new(false);

/// Flag indicando se VT-d foi inicializado
static VTD_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Próximo IOVA disponível
static NEXT_IOVA: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0x1000_0000);

/// Verifica se Intel VT-d está disponível
pub fn is_available() -> bool {
    // Se já verificamos, retornar cache
    if VTD_AVAILABLE.load(Ordering::Relaxed) || VTD_INITIALIZED.load(Ordering::Relaxed) {
        return VTD_AVAILABLE.load(Ordering::Relaxed);
    }

    // Detectar via ACPI DMAR table
    // TODO: Implementar parsing ACPI real
    // Por agora, verificar se há indicação via CPUID ou flag manual

    let available = detect_vtd();
    VTD_AVAILABLE.store(available, Ordering::Relaxed);

    available
}

/// Inicializa Intel VT-d
pub fn init() {
    if VTD_INITIALIZED.load(Ordering::Relaxed) {
        return;
    }

    if !is_available() {
        crate::kwarn!("(VT-d) Hardware não disponível");
        return;
    }

    crate::kinfo!("(VT-d) Inicializando Intel VT-d...");

    // TODO: Implementar inicialização real:
    // 1. Parsear ACPI DMAR table
    // 2. Encontrar DRHD (DMA Remapping Hardware Unit)
    // 3. Configurar root table
    // 4. Habilitar DMA remapping

    VTD_INITIALIZED.store(true, Ordering::Relaxed);
    crate::kinfo!("(VT-d) Inicializado (modo stub)");
}

/// Mapeia memória física para IOVA (I/O Virtual Address)
///
/// # Arguments
/// * `device_id` - BDF do dispositivo PCI
/// * `phys_addr` - Endereço físico a mapear
/// * `size` - Tamanho em bytes
///
/// # Returns
/// IOVA para uso pelo dispositivo
pub fn map_dma(device_id: u16, phys_addr: u64, size: usize) -> Option<u64> {
    if !VTD_INITIALIZED.load(Ordering::Relaxed) {
        return None;
    }

    // Alocar IOVA
    let iova = NEXT_IOVA.fetch_add(((size + 4095) / 4096 * 4096) as u64, Ordering::Relaxed);

    // TODO: Criar mapeamento real na page table do IOMMU
    // device_context[device_id].page_table.map(iova, phys_addr, size)

    crate::ktrace!("(VT-d) Mapeado DMA: device=", device_id as u64);
    crate::ktrace!("(VT-d)   phys=", phys_addr);
    crate::ktrace!("(VT-d)   iova=", iova);

    Some(iova)
}

/// Remove mapeamento DMA
pub fn unmap_dma(device_id: u16, iova: u64, _size: usize) {
    if !VTD_INITIALIZED.load(Ordering::Relaxed) {
        return;
    }

    // TODO: Remover mapeamento da page table do IOMMU
    // device_context[device_id].page_table.unmap(iova, size)

    crate::ktrace!("(VT-d) Desmapeado DMA: device=", device_id as u64);
    crate::ktrace!("(VT-d)   iova=", iova);
}

/// Invalida cache do IOMMU para um dispositivo
pub fn flush_iotlb(device_id: u16) {
    let _ = device_id;
    // TODO: Flush IOTLB via comando ao hardware
}

// --- Funções internas ---

fn detect_vtd() -> bool {
    // Método 1: CPUID para verificar suporte a virtualization
    // Intel: CPUID.01H:ECX[5] = VMX
    // VT-d específico requer ACPI DMAR table

    // Por agora, retornamos false (modo seguro)
    // Em produção, parsear ACPI para DMAR

    // Habilitar para teste em QEMU com: -device intel-iommu
    #[cfg(feature = "force_iommu")]
    {
        return true;
    }

    // TODO: Implementar detecção real via ACPI
    false
}
