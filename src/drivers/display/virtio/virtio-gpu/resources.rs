//! # VirtIO-GPU Resources
//!
//! Gerenciamento de recursos GPU (buffers 2D/3D).

use super::protocol::VirtioGpuFormat;
use crate::mm::{PhysAddr, VirtAddr};

// =============================================================================
// RESOURCE BACKING
// =============================================================================

/// Memória de backing para um recurso
pub struct ResourceBacking {
    pub phys_addr: PhysAddr,
    pub size: usize,
    pub virt_addr: VirtAddr,
}

// =============================================================================
// RESOURCE 2D
// =============================================================================

/// Recurso 2D alocado
pub struct Resource {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub format: VirtioGpuFormat,
    /// Buffer de backing (memória do guest)
    pub backing: Option<ResourceBacking>,
}

impl Resource {
    pub fn new(id: u32, width: u32, height: u32, format: VirtioGpuFormat) -> Self {
        Self {
            id,
            width,
            height,
            format,
            backing: None,
        }
    }

    /// Calcula stride (bytes por linha)
    pub fn stride(&self) -> u32 {
        self.width * self.format.bytes_per_pixel()
    }

    /// Tamanho total do buffer em bytes
    pub fn size(&self) -> usize {
        (self.stride() * self.height) as usize
    }

    /// Anexa backing memory
    pub fn attach_backing(&mut self, phys: PhysAddr, virt: VirtAddr, size: usize) {
        self.backing = Some(ResourceBacking {
            phys_addr: phys,
            virt_addr: virt,
            size,
        });
    }

    /// Remove backing memory
    pub fn detach_backing(&mut self) -> Option<ResourceBacking> {
        self.backing.take()
    }

    /// Verifica se tem backing
    pub fn has_backing(&self) -> bool {
        self.backing.is_some()
    }
}

// =============================================================================
// SCANOUT
// =============================================================================

/// Estado de um scanout (display virtual)
pub struct Scanout {
    pub id: u32,
    pub enabled: bool,
    pub width: u32,
    pub height: u32,
    pub resource_id: u32,
}

impl Default for Scanout {
    fn default() -> Self {
        Self {
            id: 0,
            enabled: false,
            width: 0,
            height: 0,
            resource_id: 0,
        }
    }
}

impl Scanout {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    /// Associa um recurso ao scanout
    pub fn set_resource(&mut self, resource_id: u32, width: u32, height: u32) {
        self.resource_id = resource_id;
        self.width = width;
        self.height = height;
        self.enabled = resource_id != 0;
    }

    /// Desabilita o scanout
    pub fn disable(&mut self) {
        self.resource_id = 0;
        self.enabled = false;
    }
}
