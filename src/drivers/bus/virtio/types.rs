//! # Constantes e Tipos VirtIO
//!
//! Define as constantes do protocolo VirtIO conforme especificação 1.1+.

// =============================================================================
// STATUS DO DISPOSITIVO
// =============================================================================

/// Driver reconhece o dispositivo.
pub const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 1;

/// Driver sabe como operar o dispositivo.
pub const VIRTIO_STATUS_DRIVER: u8 = 2;

/// Driver está pronto para operar.
pub const VIRTIO_STATUS_DRIVER_OK: u8 = 4;

/// Negociação de features completa.
pub const VIRTIO_STATUS_FEATURES_OK: u8 = 8;

/// Device precisa de reset.
pub const VIRTIO_STATUS_DEVICE_NEEDS_RESET: u8 = 64;

/// Falha irrecuperável.
pub const VIRTIO_STATUS_FAILED: u8 = 128;

// =============================================================================
// FEATURE BITS (DEVICE-INDEPENDENT)
// =============================================================================

/// Notificação de buffer disponível.
pub const VIRTIO_F_NOTIFY_ON_EMPTY: u64 = 1 << 24;

/// Disponibilidade de qualquer feature.
pub const VIRTIO_F_ANY_LAYOUT: u64 = 1 << 27;

/// Indirect descriptors suportados.
pub const VIRTIO_F_RING_INDIRECT_DESC: u64 = 1 << 28;

/// Event idx feature.
pub const VIRTIO_F_RING_EVENT_IDX: u64 = 1 << 29;

/// Dispositivo é VirtIO 1.0+.
pub const VIRTIO_F_VERSION_1: u64 = 1 << 32;

/// Acesso ao config space após reset.
pub const VIRTIO_F_ACCESS_PLATFORM: u64 = 1 << 33;

/// Ring packed suportado.
pub const VIRTIO_F_RING_PACKED: u64 = 1 << 34;

/// In-order requests.
pub const VIRTIO_F_IN_ORDER: u64 = 1 << 35;

/// Device-specific features acima de bit 37.
pub const VIRTIO_F_ORDER_PLATFORM: u64 = 1 << 36;

/// Single Root I/O Virtualization.
pub const VIRTIO_F_SR_IOV: u64 = 1 << 37;

/// Notification data.
pub const VIRTIO_F_NOTIFICATION_DATA: u64 = 1 << 38;

// =============================================================================
// FEATURE BITS - VIRTIO BLK
// =============================================================================

/// Capacidade máxima de segments.
pub const VIRTIO_BLK_F_SIZE_MAX: u64 = 1 << 1;

/// Capacidade máxima de seg per request.
pub const VIRTIO_BLK_F_SEG_MAX: u64 = 1 << 2;

/// Geometria de disco disponível.
pub const VIRTIO_BLK_F_GEOMETRY: u64 = 1 << 4;

/// Dispositivo é read-only.
pub const VIRTIO_BLK_F_RO: u64 = 1 << 5;

/// Tamanho de bloco disponível no config.
pub const VIRTIO_BLK_F_BLK_SIZE: u64 = 1 << 6;

/// Flush command suportado.
pub const VIRTIO_BLK_F_FLUSH: u64 = 1 << 9;

/// Topologia de dispositivo disponível.
pub const VIRTIO_BLK_F_TOPOLOGY: u64 = 1 << 10;

/// Toggle cache writeback.
pub const VIRTIO_BLK_F_CONFIG_WCE: u64 = 1 << 11;

/// Múltiplas queues.
pub const VIRTIO_BLK_F_MQ: u64 = 1 << 12;

/// Discard command suportado.
pub const VIRTIO_BLK_F_DISCARD: u64 = 1 << 13;

/// Write zeroes suportado.
pub const VIRTIO_BLK_F_WRITE_ZEROES: u64 = 1 << 14;

// =============================================================================
// FEATURE BITS - VIRTIO NET
// =============================================================================

/// Checksum offload.
pub const VIRTIO_NET_F_CSUM: u64 = 1 << 0;

/// Guest handles checksum.
pub const VIRTIO_NET_F_GUEST_CSUM: u64 = 1 << 1;

/// Control channel disponível.
pub const VIRTIO_NET_F_CTRL_VQ: u64 = 1 << 17;

/// MAC address disponível.
pub const VIRTIO_NET_F_MAC: u64 = 1 << 5;

/// GSO suportado.
pub const VIRTIO_NET_F_GSO: u64 = 1 << 6;

/// Guest handles TSO.
pub const VIRTIO_NET_F_GUEST_TSO4: u64 = 1 << 7;
pub const VIRTIO_NET_F_GUEST_TSO6: u64 = 1 << 8;
pub const VIRTIO_NET_F_GUEST_ECN: u64 = 1 << 9;
pub const VIRTIO_NET_F_GUEST_UFO: u64 = 1 << 10;

/// Host handles TSO.
pub const VIRTIO_NET_F_HOST_TSO4: u64 = 1 << 11;
pub const VIRTIO_NET_F_HOST_TSO6: u64 = 1 << 12;
pub const VIRTIO_NET_F_HOST_ECN: u64 = 1 << 13;
pub const VIRTIO_NET_F_HOST_UFO: u64 = 1 << 14;

/// Merge buffers.
pub const VIRTIO_NET_F_MRG_RXBUF: u64 = 1 << 15;

/// Status disponível.
pub const VIRTIO_NET_F_STATUS: u64 = 1 << 16;

/// Control VQ para RX mode.
pub const VIRTIO_NET_F_CTRL_RX: u64 = 1 << 18;

/// Control VQ para VLAN filtering.
pub const VIRTIO_NET_F_CTRL_VLAN: u64 = 1 << 19;

/// Múltiplas queues.
pub const VIRTIO_NET_F_MQ: u64 = 1 << 22;

// =============================================================================
// VIRTQUEUE DESCRIPTOR FLAGS
// =============================================================================

/// Próximo descritor disponível.
pub const VIRTQ_DESC_F_NEXT: u16 = 1;

/// Buffer é write-only (para dispositivo).
pub const VIRTQ_DESC_F_WRITE: u16 = 2;

/// Buffer contém lista de descritores indiretos.
pub const VIRTQ_DESC_F_INDIRECT: u16 = 4;

// =============================================================================
// TIPOS DE REQUISIÇÃO VIRTIO-BLK
// =============================================================================

/// Requisição de leitura.
pub const VIRTIO_BLK_T_IN: u32 = 0;

/// Requisição de escrita.
pub const VIRTIO_BLK_T_OUT: u32 = 1;

/// Flush de cache.
pub const VIRTIO_BLK_T_FLUSH: u32 = 4;

/// Discard de setores.
pub const VIRTIO_BLK_T_DISCARD: u32 = 11;

/// Write zeroes.
pub const VIRTIO_BLK_T_WRITE_ZEROES: u32 = 13;

// =============================================================================
// STATUS DE REQUISIÇÃO
// =============================================================================

/// Operação completada com sucesso.
pub const VIRTIO_BLK_S_OK: u8 = 0;

/// Erro de I/O.
pub const VIRTIO_BLK_S_IOERR: u8 = 1;

/// Operação não suportada.
pub const VIRTIO_BLK_S_UNSUPP: u8 = 2;
