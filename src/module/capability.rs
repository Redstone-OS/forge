//! # Module Capabilities
//!
//! Capabilities específicas para módulos de kernel.
//!
//! Estende o sistema de capabilities base (`security::capability`)
//! com tipos específicos para hardware.

use crate::security::{CapRights, CapType, Capability};

/// Tipos de capabilities que módulos podem solicitar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleCapType {
    /// Controle de GPU (MMIO + interrupts)
    GpuControl,
    /// Controle de NIC (MMIO + DMA + interrupts)
    NicControl,
    /// Controle de Storage (MMIO + DMA + interrupts)
    StorageControl,
    /// Acesso a MMIO específico
    Mmio { bus: u8, slot: u8, func: u8 },
    /// IRQ específica
    Irq { vector: u8 },
    /// Buffer DMA (requer IOMMU)
    DmaBuffer { iova: u64, size: usize },
    /// Slot de syscall (range de módulos)
    SyscallSlot { number: u16 },
    /// Timer virtual
    Timer { id: u32 },
    /// Workqueue para callbacks
    Workqueue { id: u32 },
}

/// Capability de módulo com validação extra
#[derive(Debug, Clone)]
pub struct ModuleCapability {
    /// Capability base
    pub inner: Capability,
    /// Tipo específico de módulo
    pub module_type: ModuleCapType,
    /// ID do módulo dono
    pub owner_module: u64,
    /// Se foi revogada
    pub revoked: bool,
    /// Contador de uso
    pub use_count: u64,
}

impl ModuleCapability {
    /// Cria uma nova capability de módulo
    pub fn new(
        cap_type: CapType,
        object_addr: u64,
        rights: CapRights,
        module_type: ModuleCapType,
        owner: u64,
    ) -> Self {
        Self {
            inner: Capability::new(cap_type, object_addr, rights),
            module_type,
            owner_module: owner,
            revoked: false,
            use_count: 0,
        }
    }

    /// Verifica se capability é válida para uso
    pub fn is_valid(&self) -> bool {
        !self.revoked && self.inner.object_type != CapType::Null
    }

    /// Verifica se tem os direitos necessários
    pub fn check(&self, required: CapRights) -> bool {
        self.is_valid() && self.inner.check(required)
    }

    /// Registra um uso da capability
    pub fn record_use(&mut self) {
        self.use_count += 1;
    }

    /// Revoga a capability
    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}

/// Gerenciador de capabilities de módulos
#[allow(dead_code)]
pub struct ModuleCapabilityManager {
    /// Próximo ID de capability
    next_id: u64,
    /// Capabilities ativas
    capabilities: alloc::vec::Vec<ModuleCapability>,
    /// Limite de capabilities por módulo
    max_per_module: usize,
}

impl ModuleCapabilityManager {
    /// Cria um novo gerenciador
    pub const fn new() -> Self {
        Self {
            next_id: 1,
            capabilities: alloc::vec::Vec::new(),
            max_per_module: 64,
        }
    }

    /// Solicita uma capability para um módulo
    pub fn request(
        &mut self,
        module_id: u64,
        cap_type: ModuleCapType,
    ) -> Result<&ModuleCapability, CapabilityError> {
        // Verificar limite por módulo
        let module_cap_count = self
            .capabilities
            .iter()
            .filter(|c| c.owner_module == module_id && !c.revoked)
            .count();

        if module_cap_count >= self.max_per_module {
            return Err(CapabilityError::LimitReached);
        }

        // Verificar se a capability pode ser concedida
        if !self.can_grant(&cap_type) {
            return Err(CapabilityError::Denied);
        }

        // Criar capability baseada no tipo
        let (base_type, addr, rights) = self.resolve_cap_type(&cap_type)?;

        let cap = ModuleCapability::new(base_type, addr, rights, cap_type, module_id);

        self.capabilities.push(cap);

        Ok(self.capabilities.last().unwrap())
    }

    /// Revoga todas capabilities de um módulo
    pub fn revoke_all(&mut self, module_id: u64) {
        for cap in &mut self.capabilities {
            if cap.owner_module == module_id {
                cap.revoke();
            }
        }
    }

    /// Revoga uma capability específica
    pub fn revoke(&mut self, cap_id: u64) {
        if let Some(cap) = self
            .capabilities
            .iter_mut()
            .find(|c| c.inner.object_addr == cap_id)
        {
            cap.revoke();
        }
    }

    /// Verifica se uma capability pode ser concedida
    fn can_grant(&self, cap_type: &ModuleCapType) -> bool {
        match cap_type {
            ModuleCapType::DmaBuffer { .. } => {
                // DMA requer IOMMU
                crate::module::has_iommu()
            }
            ModuleCapType::Irq { vector } => {
                // Verificar se IRQ não está em uso por componente crítico
                *vector >= 32 && *vector < 48 // Apenas IRQs de devices
            }
            ModuleCapType::SyscallSlot { number } => {
                // Syscalls de módulo são 768-1023
                *number >= 768 && *number < 1024
            }
            _ => true,
        }
    }

    /// Resolve tipo de capability para parâmetros base
    fn resolve_cap_type(
        &self,
        cap_type: &ModuleCapType,
    ) -> Result<(CapType, u64, CapRights), CapabilityError> {
        match cap_type {
            ModuleCapType::GpuControl => Ok((
                CapType::Device,
                0, // Será preenchido com endereço real
                CapRights::READ | CapRights::WRITE,
            )),
            ModuleCapType::NicControl => {
                Ok((CapType::Device, 0, CapRights::READ | CapRights::WRITE))
            }
            ModuleCapType::StorageControl => {
                Ok((CapType::Device, 0, CapRights::READ | CapRights::WRITE))
            }
            ModuleCapType::Mmio { .. } => {
                Ok((CapType::Memory, 0, CapRights::READ | CapRights::WRITE))
            }
            ModuleCapType::Irq { vector } => Ok((CapType::Irq, *vector as u64, CapRights::READ)),
            ModuleCapType::DmaBuffer { iova, .. } => {
                Ok((CapType::Memory, *iova, CapRights::READ | CapRights::WRITE))
            }
            ModuleCapType::SyscallSlot { number } => Ok((
                CapType::Task, // Reutilizando para syscall
                *number as u64,
                CapRights::CALL,
            )),
            ModuleCapType::Timer { id } => Ok((
                CapType::Task,
                *id as u64,
                CapRights::READ | CapRights::WRITE,
            )),
            ModuleCapType::Workqueue { id } => Ok((CapType::Task, *id as u64, CapRights::WRITE)),
        }
    }
}

/// Erros de capability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityError {
    /// Capability negada
    Denied,
    /// Limite de capabilities atingido
    LimitReached,
    /// Recurso não disponível
    ResourceUnavailable,
    /// IOMMU requerido
    IommuRequired,
}
