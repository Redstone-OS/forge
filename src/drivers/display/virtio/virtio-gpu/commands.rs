//! # VirtIO-GPU Command Structures
//!
//! Estruturas de comandos e respostas do protocolo VirtIO-GPU.

use super::protocol::{CtrlType, VirtioGpuFormat, MAX_SCANOUTS, VIRTIO_GPU_FLAG_FENCE};

// =============================================================================
// ESTRUTURAS BASE
// =============================================================================

/// Header comum para todos os comandos
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CtrlHeader {
    pub ctrl_type: u32,
    pub flags: u32,
    pub fence_id: u64,
    pub ctx_id: u32,
    pub ring_idx: u8,
    pub padding: [u8; 3],
}

/// Retângulo para operações de display
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

// =============================================================================
// DISPLAY INFO
// =============================================================================

/// Informação de um display (scanout)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DisplayOne {
    pub r: Rect,
    pub enabled: u32,
    pub flags: u32,
}

/// Resposta GET_DISPLAY_INFO
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RespDisplayInfo {
    pub hdr: CtrlHeader,
    pub pmodes: [DisplayOne; MAX_SCANOUTS],
}

// =============================================================================
// RESOURCE COMMANDS
// =============================================================================

/// Comando RESOURCE_CREATE_2D
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ResourceCreate2d {
    pub hdr: CtrlHeader,
    pub resource_id: u32,
    pub format: u32,
    pub width: u32,
    pub height: u32,
}

/// Entry para RESOURCE_ATTACH_BACKING
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MemEntry {
    pub addr: u64,
    pub length: u32,
    pub padding: u32,
}

/// Comando RESOURCE_ATTACH_BACKING
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ResourceAttachBacking {
    pub hdr: CtrlHeader,
    pub resource_id: u32,
    pub nr_entries: u32,
}

// =============================================================================
// SCANOUT COMMANDS
// =============================================================================

/// Comando SET_SCANOUT
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SetScanout {
    pub hdr: CtrlHeader,
    pub r: Rect,
    pub scanout_id: u32,
    pub resource_id: u32,
}

/// Comando RESOURCE_FLUSH
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ResourceFlush {
    pub hdr: CtrlHeader,
    pub r: Rect,
    pub resource_id: u32,
    pub padding: u32,
}

/// Comando TRANSFER_TO_HOST_2D
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TransferToHost2d {
    pub hdr: CtrlHeader,
    pub r: Rect,
    pub offset: u64,
    pub resource_id: u32,
    pub padding: u32,
}

// =============================================================================
// CURSOR COMMANDS
// =============================================================================

/// Posição do cursor
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CursorPos {
    pub scanout_id: u32,
    pub x: u32,
    pub y: u32,
    pub padding: u32,
}

/// Comando UPDATE_CURSOR
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UpdateCursor {
    pub hdr: CtrlHeader,
    pub pos: CursorPos,
    pub resource_id: u32,
    pub hot_x: u32,
    pub hot_y: u32,
    pub padding: u32,
}

// =============================================================================
// BUILDERS
// =============================================================================

impl CtrlHeader {
    pub fn new(ctrl_type: CtrlType) -> Self {
        Self {
            ctrl_type: ctrl_type as u32,
            flags: 0,
            fence_id: 0,
            ctx_id: 0,
            ring_idx: 0,
            padding: [0; 3],
        }
    }

    pub fn with_fence(mut self, fence_id: u64) -> Self {
        self.flags |= VIRTIO_GPU_FLAG_FENCE;
        self.fence_id = fence_id;
        self
    }
}

impl ResourceCreate2d {
    pub fn new(resource_id: u32, format: VirtioGpuFormat, width: u32, height: u32) -> Self {
        Self {
            hdr: CtrlHeader::new(CtrlType::ResourceCreate2d),
            resource_id,
            format: format as u32,
            width,
            height,
        }
    }
}

impl SetScanout {
    pub fn new(scanout_id: u32, resource_id: u32, width: u32, height: u32) -> Self {
        Self {
            hdr: CtrlHeader::new(CtrlType::SetScanout),
            r: Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            scanout_id,
            resource_id,
        }
    }

    pub fn disable(scanout_id: u32) -> Self {
        Self {
            hdr: CtrlHeader::new(CtrlType::SetScanout),
            r: Rect::default(),
            scanout_id,
            resource_id: 0,
        }
    }
}

impl ResourceFlush {
    pub fn new(resource_id: u32, width: u32, height: u32) -> Self {
        Self {
            hdr: CtrlHeader::new(CtrlType::ResourceFlush),
            r: Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            resource_id,
            padding: 0,
        }
    }
}

impl TransferToHost2d {
    pub fn new(resource_id: u32, width: u32, height: u32) -> Self {
        Self {
            hdr: CtrlHeader::new(CtrlType::TransferToHost2d),
            r: Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            offset: 0,
            resource_id,
            padding: 0,
        }
    }
}

impl ResourceAttachBacking {
    pub fn new(resource_id: u32, nr_entries: u32) -> Self {
        Self {
            hdr: CtrlHeader::new(CtrlType::ResourceAttachBacking),
            resource_id,
            nr_entries,
        }
    }
}
