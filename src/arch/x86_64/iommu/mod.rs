//! # IOMMU Abstraction Layer
//!
//! Abstração de IOMMU para isolamento de DMA.
//!
//! ## Suporte
//! - Intel VT-d (DMAR)
//! - AMD-Vi (IVRS)
//!
//! ## Propósito
//! Sem IOMMU, dispositivos PCI podem fazer DMA para qualquer
//! endereço físico, quebrando isolamento de memória.
//! Com IOMMU, cada dispositivo tem sua própria page table de DMA.

pub mod intel_vtd;

/// Verifica se IOMMU está disponível no sistema
pub fn is_available() -> bool {
    intel_vtd::is_available()
}

/// Inicializa o IOMMU (se disponível)
pub fn init() -> bool {
    if intel_vtd::is_available() {
        intel_vtd::init();
        return true;
    }
    false
}

/// Tipo de IOMMU detectado
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IommuType {
    /// Não disponível
    None,
    /// Intel VT-d
    IntelVtd,
    /// AMD-Vi
    AmdVi,
}

/// Retorna o tipo de IOMMU disponível
pub fn get_type() -> IommuType {
    if intel_vtd::is_available() {
        return IommuType::IntelVtd;
    }
    IommuType::None
}

/// Buffer DMA protegido por IOMMU
pub struct DmaBuffer {
    /// Endereço físico do buffer
    pub phys_addr: u64,
    /// Endereço virtual do buffer (para CPU)
    pub virt_addr: u64,
    /// Endereço de I/O (IOVA) para dispositivos
    pub iova: u64,
    /// Tamanho do buffer
    pub size: usize,
    /// ID do dispositivo dono
    pub device_id: u16,
}

impl DmaBuffer {
    /// Aloca um buffer DMA protegido
    ///
    /// # Arguments
    /// * `size` - Tamanho em bytes
    /// * `device_id` - ID do dispositivo PCI (BDF)
    ///
    /// # Returns
    /// Buffer alocado ou None se falhou
    pub fn alloc(size: usize, device_id: u16) -> Option<Self> {
        if !is_available() {
            crate::kwarn!("(IOMMU) Tentativa de alocar DMA sem IOMMU!");
            return None;
        }

        // Alocar memória física via PMM
        let frame = crate::mm::pmm::FRAME_ALLOCATOR.lock().allocate_frame()?;
        let phys_addr = frame.addr();

        // Converter para virtual
        let phys = crate::mm::addr::PhysAddr::new(phys_addr);
        let virt = crate::mm::addr::phys_to_virt(phys);
        let virt_addr = virt.as_u64();

        // Mapear no IOMMU
        let iova = intel_vtd::map_dma(device_id, phys_addr, size)?;

        Some(Self {
            phys_addr,
            virt_addr,
            iova,
            size,
            device_id,
        })
    }

    /// Libera o buffer DMA
    pub fn free(self) {
        // Desmapear do IOMMU
        intel_vtd::unmap_dma(self.device_id, self.iova, self.size);
        // Nota: liberação de frames deve ser feita quando API estiver estável
    }

    /// Retorna ponteiro para escrita (CPU)
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.virt_addr as *mut u8
    }

    /// Retorna slice mutável do buffer
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.virt_addr as *mut u8, self.size) }
    }
}
