//! # Module ABI
//!
//! Interface estável para módulos de kernel.
//!
//! ## Filosofia
//! Esta é a ÚNICA interface que módulos podem usar para interagir com o kernel.
//! Qualquer outro acesso é bloqueado pelo sandbox.
//!
//! ## Estabilidade
//! A ABI é versionada. Mudanças quebram compatibilidade apenas em major versions.

use super::capability::{ModuleCapType, ModuleCapability};
use super::ModuleError;

/// Versão atual da Module ABI
pub const ABI_VERSION: u32 = 1;

/// Magic number para identificar módulos válidos
pub const MODULE_MAGIC: u32 = 0x524D4F44; // "RMOD" em little-endian

/// Informações estáticas de um módulo
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Magic number (deve ser MODULE_MAGIC)
    pub magic: u32,
    /// Versão da ABI usada
    pub abi_version: u32,
    /// Nome do módulo (null-terminated)
    pub name: [u8; 64],
    /// Versão do módulo (major.minor.patch)
    pub version: (u16, u16, u16),
    /// Flags do módulo
    pub flags: ModuleFlags,
    /// Capabilities requeridas (bitmask)
    pub required_caps: u64,
    /// Capabilities opcionais (bitmask)
    pub optional_caps: u64,
    /// Offset do entry point (relativo ao início do módulo)
    pub init_offset: u64,
    /// Offset da função de saída
    pub exit_offset: u64,
    /// Hash do certificado do autor
    pub author_cert_hash: [u8; 32],
}

impl ModuleInfo {
    /// Verifica se as informações são válidas
    pub fn is_valid(&self) -> bool {
        self.magic == MODULE_MAGIC && self.abi_version <= ABI_VERSION
    }

    /// Obtém nome como string
    pub fn name_str(&self) -> &str {
        let null_pos = self.name.iter().position(|&b| b == 0).unwrap_or(64);
        core::str::from_utf8(&self.name[..null_pos]).unwrap_or("???")
    }
}

bitflags::bitflags! {
    /// Flags do módulo
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ModuleFlags: u32 {
        /// Módulo requer IOMMU para DMA
        const REQUIRES_IOMMU = 1 << 0;
        /// Módulo é crítico (panic se falhar)
        const CRITICAL = 1 << 1;
        /// Módulo pode ser recarregado em hot-reload
        const HOT_RELOAD = 1 << 2;
        /// Módulo é um driver de GPU
        const GPU_DRIVER = 1 << 3;
        /// Módulo é um driver de rede
        const NET_DRIVER = 1 << 4;
        /// Módulo é um driver de storage
        const STORAGE_DRIVER = 1 << 5;
    }
}

/// Funções que o kernel exporta para módulos
pub struct ModuleAbi;

impl ModuleAbi {
    // =========================================================================
    // MEMÓRIA
    // =========================================================================

    /// Aloca buffer DMA seguro (via IOMMU)
    pub fn alloc_dma(size: usize, cap: &ModuleCapability) -> Result<(u64, u64), ModuleError> {
        // Verificar capability
        if !matches!(cap.module_type, ModuleCapType::DmaBuffer { .. }) {
            return Err(ModuleError::CapabilityDenied);
        }

        if cap.revoked {
            return Err(ModuleError::CapabilityDenied);
        }

        // Verificar IOMMU
        if !super::has_iommu() {
            return Err(ModuleError::IommuRequired);
        }

        // TODO: Alocar via IOMMU quando API estiver estável
        let _ = size;
        Err(ModuleError::InternalError)
    }

    /// Libera buffer DMA
    pub fn free_dma(_phys: u64, _size: usize) {
        // TODO: Implementar quando API estiver estável
    }

    // =========================================================================
    // MMIO
    // =========================================================================

    /// Mapeia região MMIO do dispositivo
    pub fn map_mmio(cap: &ModuleCapability) -> Result<*mut u8, ModuleError> {
        if !matches!(cap.module_type, ModuleCapType::Mmio { .. }) {
            return Err(ModuleError::CapabilityDenied);
        }

        if cap.revoked {
            return Err(ModuleError::CapabilityDenied);
        }

        // TODO: Mapear MMIO real
        Ok(core::ptr::null_mut())
    }

    // =========================================================================
    // IRQ
    // =========================================================================

    /// Registra handler de IRQ
    pub fn request_irq(irq: u8, handler: fn(), cap: &ModuleCapability) -> Result<(), ModuleError> {
        if !matches!(cap.module_type, ModuleCapType::Irq { vector } if vector == irq) {
            return Err(ModuleError::CapabilityDenied);
        }

        if cap.revoked {
            return Err(ModuleError::CapabilityDenied);
        }

        // TODO: Registrar handler
        let _ = handler;

        Ok(())
    }

    /// Remove handler de IRQ
    pub fn free_irq(_irq: u8) {
        // TODO: Remover handler
    }

    // =========================================================================
    // SYSCALLS
    // =========================================================================

    /// Registra syscall em slot alocado
    pub fn register_syscall(handler: fn(), cap: &ModuleCapability) -> Result<u16, ModuleError> {
        if !matches!(cap.module_type, ModuleCapType::SyscallSlot { .. }) {
            return Err(ModuleError::CapabilityDenied);
        }

        if cap.revoked {
            return Err(ModuleError::CapabilityDenied);
        }

        if let ModuleCapType::SyscallSlot { number } = cap.module_type {
            // TODO: Registrar syscall
            let _ = handler;
            Ok(number)
        } else {
            Err(ModuleError::InternalError)
        }
    }

    // =========================================================================
    // TIMERS
    // =========================================================================

    /// Cria timer virtual
    pub fn create_timer(_callback: fn(), _interval_ms: u64) -> Result<u32, ModuleError> {
        // TODO: Criar timer
        Ok(0)
    }

    /// Cancela timer
    pub fn cancel_timer(_timer_id: u32) {
        // TODO: Cancelar timer
    }

    // =========================================================================
    // WORKQUEUE
    // =========================================================================

    /// Agenda trabalho na workqueue
    pub fn schedule_work(_work: fn()) -> Result<(), ModuleError> {
        // TODO: Agendar trabalho
        Ok(())
    }

    // =========================================================================
    // LOGGING
    // =========================================================================

    /// Log (sempre público, nunca suprimível)
    pub fn log(level: LogLevel, _msg: &str) {
        match level {
            LogLevel::Error => crate::kerror!("(Module) Log"),
            LogLevel::Warn => crate::kwarn!("(Module) Log"),
            LogLevel::Info => crate::kinfo!("(Module) Log"),
            LogLevel::Debug => crate::kdebug!("(Module) Log"),
            LogLevel::Trace => crate::ktrace!("(Module) Log"),
        }
    }

    // =========================================================================
    // HEALTHCHECK
    // =========================================================================

    /// Reporta heartbeat ao watchdog
    pub fn heartbeat(module_id: u64) {
        super::SUPERVISOR
            .lock()
            .watchdog
            .heartbeat(super::ModuleId::new(module_id));
    }

    /// Reporta falha recuperável
    pub fn report_fault(module_id: u64) {
        super::SUPERVISOR
            .lock()
            .report_fault(super::ModuleId::new(module_id));
    }
}

/// Níveis de log
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
