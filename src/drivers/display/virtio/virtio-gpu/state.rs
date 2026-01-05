//! # VirtIO-GPU State Management
//!
//! Estado global do dispositivo VirtIO-GPU.

use super::protocol::MAX_SCANOUTS;
use super::resources::{Resource, Scanout};
use crate::sync::Spinlock;
use alloc::vec::Vec;

// =============================================================================
// DEVICE STATE
// =============================================================================

/// Estado do dispositivo VirtIO-GPU
pub struct VirtioGpuState {
    /// Features negociadas
    pub features: u64,
    /// Recursos alocados
    pub resources: Vec<Resource>,
    /// Próximo resource ID
    pub next_resource_id: u32,
    /// Scanouts (displays virtuais)
    pub scanouts: [Scanout; MAX_SCANOUTS],
    /// Número de scanouts ativos
    pub num_scanouts: u32,
    /// Fence ID atual
    pub fence_id: u64,
}

impl VirtioGpuState {
    pub fn new() -> Self {
        Self {
            features: 0,
            resources: Vec::new(),
            next_resource_id: 1, // 0 é inválido
            scanouts: core::array::from_fn(|i| Scanout::new(i as u32)),
            num_scanouts: 0,
            fence_id: 0,
        }
    }

    /// Aloca um novo resource ID
    pub fn alloc_resource_id(&mut self) -> u32 {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        id
    }

    /// Encontra recurso por ID
    pub fn find_resource(&self, id: u32) -> Option<&Resource> {
        self.resources.iter().find(|r| r.id == id)
    }

    /// Encontra recurso por ID (mutável)
    pub fn find_resource_mut(&mut self, id: u32) -> Option<&mut Resource> {
        self.resources.iter_mut().find(|r| r.id == id)
    }

    /// Remove recurso por ID
    pub fn remove_resource(&mut self, id: u32) -> Option<Resource> {
        if let Some(pos) = self.resources.iter().position(|r| r.id == id) {
            Some(self.resources.remove(pos))
        } else {
            None
        }
    }

    /// Próximo fence ID
    pub fn next_fence_id(&mut self) -> u64 {
        self.fence_id += 1;
        self.fence_id
    }

    /// Verifica se recurso existe
    pub fn resource_exists(&self, id: u32) -> bool {
        self.find_resource(id).is_some()
    }
}

// =============================================================================
// GLOBAL STATE
// =============================================================================

/// Estado global protegido por lock
pub static GPU_STATE: Spinlock<Option<VirtioGpuState>> = Spinlock::new(None);

/// Inicializa o estado do VirtIO-GPU
pub fn init_state() {
    let mut state = GPU_STATE.lock();
    if state.is_none() {
        *state = Some(VirtioGpuState::new());
        crate::kdebug!("(VirtIO-GPU) Estado inicializado");
    }
}

/// Verifica se o recurso existe
pub fn resource_exists(id: u32) -> bool {
    GPU_STATE
        .lock()
        .as_ref()
        .map_or(false, |s| s.resource_exists(id))
}

/// Obtém informações do display primário
pub fn get_primary_display_info() -> Option<(u32, u32)> {
    let state = GPU_STATE.lock();
    state.as_ref().and_then(|s| {
        if s.num_scanouts > 0 && s.scanouts[0].enabled {
            Some((s.scanouts[0].width, s.scanouts[0].height))
        } else {
            None
        }
    })
}
