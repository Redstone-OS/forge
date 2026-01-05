//! # VirtIO-GPU Protocol Definitions
//!
//! Constantes e tipos do protocolo VirtIO-GPU.
//! Spec: OASIS VirtIO v1.2 - Section 5.7

#![allow(dead_code)]

// =============================================================================
// CONSTANTES DO PROTOCOLO
// =============================================================================

/// Máximo de scanouts (displays virtuais)
pub const MAX_SCANOUTS: usize = 16;

/// Tamanho do cabeçalho de comando/resposta
pub const CTRL_HDR_SIZE: usize = 24;

// Feature flags
pub const VIRTIO_GPU_F_VIRGL: u64 = 1 << 0;
pub const VIRTIO_GPU_F_EDID: u64 = 1 << 1;
pub const VIRTIO_GPU_F_RESOURCE_UUID: u64 = 1 << 2;
pub const VIRTIO_GPU_F_RESOURCE_BLOB: u64 = 1 << 3;
pub const VIRTIO_GPU_F_CONTEXT_INIT: u64 = 1 << 4;

// Control flags
pub const VIRTIO_GPU_FLAG_FENCE: u32 = 1 << 0;

// =============================================================================
// TIPOS DE COMANDO
// =============================================================================

/// Tipos de comando VirtIO-GPU
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CtrlType {
    // 2D commands
    GetDisplayInfo = 0x0100,
    ResourceCreate2d = 0x0101,
    ResourceUnref = 0x0102,
    SetScanout = 0x0103,
    ResourceFlush = 0x0104,
    TransferToHost2d = 0x0105,
    ResourceAttachBacking = 0x0106,
    ResourceDetachBacking = 0x0107,
    GetCapsetInfo = 0x0108,
    GetCapset = 0x0109,
    GetEdid = 0x010A,
    ResourceAssignUuid = 0x010B,
    ResourceCreateBlob = 0x010C,
    SetScanoutBlob = 0x010D,

    // Cursor commands
    UpdateCursor = 0x0300,
    MoveCursor = 0x0301,

    // Success responses
    RespOkNoData = 0x1100,
    RespOkDisplayInfo = 0x1101,
    RespOkCapsetInfo = 0x1102,
    RespOkCapset = 0x1103,
    RespOkEdid = 0x1104,
    RespOkResourceUuid = 0x1105,
    RespOkMapInfo = 0x1106,

    // Error responses
    RespErrUnspec = 0x1200,
    RespErrOutOfMemory = 0x1201,
    RespErrInvalidScanoutId = 0x1202,
    RespErrInvalidResourceId = 0x1203,
    RespErrInvalidContextId = 0x1204,
    RespErrInvalidParameter = 0x1205,
}

// =============================================================================
// FORMATOS DE PIXEL
// =============================================================================

/// Formatos de pixel suportados
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtioGpuFormat {
    B8G8R8A8Unorm = 1,
    B8G8R8X8Unorm = 2,
    A8R8G8B8Unorm = 3,
    X8R8G8B8Unorm = 4,
    R8G8B8A8Unorm = 67,
    X8B8G8R8Unorm = 68,
    A8B8G8R8Unorm = 121,
    R8G8B8X8Unorm = 134,
}

impl VirtioGpuFormat {
    pub fn bytes_per_pixel(self) -> u32 {
        4 // Todos os formatos suportados são 32bpp
    }
}
