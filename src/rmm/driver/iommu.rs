//! # IOMMU Abstraction
//!
//! Abstração para IOMMU (Input/Output Memory Management Unit).
//!
//! ## Suporte
//!
//! - **Intel VT-d**: Via DMAR ACPI table
//! - **AMD-Vi**: Via IVRS ACPI table
//! - **NoIommu**: Fallback para hardware sem IOMMU
//!
//! ## Conceitos
//!
//! ### Domain
//! Espaço de endereçamento isolado para um dispositivo ou grupo.
//! Análogo a um address space de processo, mas para DMA.
//!
//! ### Passthrough
//! Modo onde IOMMU mapeia 1:1 (identity mapping).
//! Menos seguro mas mais simples.
//!
//! ## Lifetime Management
//!
//! CRÍTICO: Quando um driver falha ou é descarregado, todos os
//! mapeamentos IOMMU do dispositivo devem ser removidos para
//! evitar DMA attacks.
//!
//! ```text
//! Driver Load:
//! 1. Cria IommuDomain
//! 2. Mapeia buffers necessários
//!
//! Driver Unload/Crash:
//! 1. Invalida todos os mapeamentos do domain
//! 2. Flush IOTLB
//! 3. Libera domain
//! ```

use crate::rmm::addr::PhysAddr;
use crate::rmm::error::{RmmError, RmmResult};
use crate::sync::Spinlock;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Flag indicando se IOMMU está disponível
static IOMMU_AVAILABLE: Spinlock<bool> = Spinlock::new(false);

/// Driver IOMMU atual
static IOMMU_DRIVER: Spinlock<Option<IommuDriver>> = Spinlock::new(None);

/// Contador de domains
static NEXT_DOMAIN_ID: Spinlock<u32> = Spinlock::new(1);

// =============================================================================
// IommuOps Trait
// =============================================================================

/// Trait para operações IOMMU
pub trait IommuOps: Send + Sync {
    /// Nome do driver
    fn name(&self) -> &'static str;

    /// Cria novo domain
    fn create_domain(&self) -> RmmResult<u32>;

    /// Destroi domain
    fn destroy_domain(&self, domain_id: u32) -> RmmResult<()>;

    /// Mapeia região de memória
    fn map(
        &self,
        domain_id: u32,
        iova: u64,
        phys: PhysAddr,
        size: usize,
        flags: IommuFlags,
    ) -> RmmResult<()>;

    /// Remove mapeamento
    fn unmap(&self, domain_id: u32, iova: u64, size: usize) -> RmmResult<()>;

    /// Flush IOTLB
    fn flush(&self, domain_id: u32) -> RmmResult<()>;

    /// Attach dispositivo ao domain
    fn attach(&self, domain_id: u32, device_id: u32) -> RmmResult<()>;

    /// Detach dispositivo do domain
    fn detach(&self, domain_id: u32, device_id: u32) -> RmmResult<()>;
}

// =============================================================================
// IommuFlags
// =============================================================================

/// Flags para mapeamento IOMMU
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct IommuFlags(u32);

impl IommuFlags {
    /// Leitura permitida
    pub const READ: Self = Self(1 << 0);
    /// Escrita permitida
    pub const WRITE: Self = Self(1 << 1);
    /// Cache habilitado
    pub const CACHE: Self = Self(1 << 2);
    /// No snoop (bypass cache coherency)
    pub const NOSNOOP: Self = Self(1 << 3);

    /// Read + Write
    pub const RW: Self = Self(Self::READ.0 | Self::WRITE.0);

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

// =============================================================================
// NoIommu (Fallback)
// =============================================================================

/// Implementação fallback sem IOMMU
///
/// Usa identity mapping (IOVA == PA).
/// ATENÇÃO: Sem proteção de DMA!
pub struct NoIommu;

impl IommuOps for NoIommu {
    fn name(&self) -> &'static str {
        "NoIommu"
    }

    fn create_domain(&self) -> RmmResult<u32> {
        let mut next = NEXT_DOMAIN_ID.lock();
        let id = *next;
        *next += 1;
        Ok(id)
    }

    fn destroy_domain(&self, _domain_id: u32) -> RmmResult<()> {
        Ok(())
    }

    fn map(
        &self,
        _domain_id: u32,
        _iova: u64,
        _phys: PhysAddr,
        _size: usize,
        _flags: IommuFlags,
    ) -> RmmResult<()> {
        // Identity mapping - nada a fazer
        Ok(())
    }

    fn unmap(&self, _domain_id: u32, _iova: u64, _size: usize) -> RmmResult<()> {
        Ok(())
    }

    fn flush(&self, _domain_id: u32) -> RmmResult<()> {
        Ok(())
    }

    fn attach(&self, _domain_id: u32, _device_id: u32) -> RmmResult<()> {
        Ok(())
    }

    fn detach(&self, _domain_id: u32, _device_id: u32) -> RmmResult<()> {
        Ok(())
    }
}

// =============================================================================
// IommuDriver
// =============================================================================

/// Tipo de driver IOMMU
enum IommuDriver {
    None(NoIommu),
    // Futuro:
    // IntelVtd(VtdDriver),
    // AmdVi(AmdViDriver),
}

impl IommuDriver {
    fn ops(&self) -> &dyn IommuOps {
        match self {
            Self::None(d) => d,
        }
    }
}

// =============================================================================
// IommuDomain
// =============================================================================

/// Domain IOMMU para um driver/dispositivo
///
/// Encapsula mapeamentos e garante cleanup no drop.
pub struct IommuDomain {
    /// ID do domain
    id: u32,
    /// Device IDs attached
    devices: Vec<u32>,
    /// Mapeamentos ativos: IOVA -> (PhysAddr, size)
    mappings: BTreeMap<u64, (PhysAddr, usize)>,
    /// Driver ID (para logging/audit)
    driver_id: u32,
}

impl IommuDomain {
    /// Cria novo domain
    pub fn new(driver_id: u32) -> RmmResult<Self> {
        let id = with_iommu(|iommu| iommu.create_domain())?;

        Ok(Self {
            id,
            devices: Vec::new(),
            mappings: BTreeMap::new(),
            driver_id,
        })
    }

    /// ID do domain
    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Attach dispositivo
    pub fn attach(&mut self, device_id: u32) -> RmmResult<()> {
        with_iommu(|iommu| iommu.attach(self.id, device_id))?;
        self.devices.push(device_id);
        Ok(())
    }

    /// Detach dispositivo
    pub fn detach(&mut self, device_id: u32) -> RmmResult<()> {
        with_iommu(|iommu| iommu.detach(self.id, device_id))?;
        self.devices.retain(|&d| d != device_id);
        Ok(())
    }

    /// Mapeia região
    ///
    /// IOVA (I/O Virtual Address) é o endereço que o dispositivo usa.
    /// Geralmente IOVA == PhysAddr para identity mapping.
    pub fn map(
        &mut self,
        iova: u64,
        phys: PhysAddr,
        size: usize,
        flags: IommuFlags,
    ) -> RmmResult<()> {
        with_iommu(|iommu| iommu.map(self.id, iova, phys, size, flags))?;
        self.mappings.insert(iova, (phys, size));
        Ok(())
    }

    /// Mapeia com identity (IOVA == PA)
    pub fn map_identity(
        &mut self,
        phys: PhysAddr,
        size: usize,
        flags: IommuFlags,
    ) -> RmmResult<()> {
        self.map(phys.as_u64(), phys, size, flags)
    }

    /// Remove mapeamento
    pub fn unmap(&mut self, iova: u64) -> RmmResult<()> {
        if let Some((_, size)) = self.mappings.remove(&iova) {
            with_iommu(|iommu| iommu.unmap(self.id, iova, size))?;
        }
        Ok(())
    }

    /// Remove todos os mapeamentos
    pub fn unmap_all(&mut self) -> RmmResult<()> {
        let mappings: Vec<_> = self.mappings.keys().copied().collect();
        for iova in mappings {
            self.unmap(iova)?;
        }
        Ok(())
    }

    /// Flush IOTLB
    pub fn flush(&self) -> RmmResult<()> {
        with_iommu(|iommu| iommu.flush(self.id))
    }

    /// Número de mapeamentos ativos
    pub fn mapping_count(&self) -> usize {
        self.mappings.len()
    }

    /// Número de dispositivos attached
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
}

impl Drop for IommuDomain {
    fn drop(&mut self) {
        // CRÍTICO: Limpa todos os mapeamentos ao dropar
        // Isso previne DMA attacks de dispositivos maliciosos

        // Detach todos os dispositivos
        for device_id in self.devices.clone() {
            let _ = with_iommu(|iommu| iommu.detach(self.id, device_id));
        }

        // Remove todos os mapeamentos
        for (iova, (_, size)) in self.mappings.iter() {
            let _ = with_iommu(|iommu| iommu.unmap(self.id, *iova, *size));
        }

        // Flush
        let _ = with_iommu(|iommu| iommu.flush(self.id));

        // Destroi domain
        let _ = with_iommu(|iommu| iommu.destroy_domain(self.id));

        #[cfg(debug_assertions)]
        crate::kinfo!(
            "(IOMMU) Domain {} destroyed (driver {})",
            self.id,
            self.driver_id
        );
    }
}

// =============================================================================
// API Global
// =============================================================================

/// Detecta e inicializa IOMMU
pub fn detect() {
    // TODO: Detectar via ACPI DMAR (Intel) ou IVRS (AMD)

    // Por enquanto, usa fallback
    *IOMMU_DRIVER.lock() = Some(IommuDriver::None(NoIommu));
    *IOMMU_AVAILABLE.lock() = false; // NoIommu não conta como "disponível"

    crate::kinfo!("(IOMMU) Usando fallback NoIommu (identity mapping)");
}

/// Verifica se IOMMU real está disponível
pub fn is_available() -> bool {
    *IOMMU_AVAILABLE.lock()
}

/// Retorna nome do driver IOMMU atual
pub fn driver_name() -> &'static str {
    let driver = IOMMU_DRIVER.lock();
    match driver.as_ref() {
        Some(d) => d.ops().name(),
        None => "None",
    }
}

/// Executa operação com o IOMMU driver
fn with_iommu<F, R>(f: F) -> RmmResult<R>
where
    F: FnOnce(&dyn IommuOps) -> RmmResult<R>,
{
    let driver = IOMMU_DRIVER.lock();
    match driver.as_ref() {
        Some(d) => f(d.ops()),
        None => Err(RmmError::NotInitialized),
    }
}

/// Cria novo domain (helper)
pub fn create_domain(driver_id: u32) -> RmmResult<IommuDomain> {
    IommuDomain::new(driver_id)
}
