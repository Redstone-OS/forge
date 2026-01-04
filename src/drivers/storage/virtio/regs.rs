//! # Registradores e Constantes VirtIO Block
//!
//! Definições de todas as constantes do VirtIO Block Device.

/// Tamanho padrão de setor
pub const SECTOR_SIZE: usize = 512;

/// Tipos de operação VirtIO Block
pub mod blk_type {
    /// Leitura
    pub const IN: u32 = 0;
    /// Escrita
    pub const OUT: u32 = 1;
    /// Flush (barrier)
    pub const FLUSH: u32 = 4;
    /// Get device ID
    pub const GET_ID: u32 = 8;
}

/// Status de resposta VirtIO Block
pub mod blk_status {
    /// Operação bem-sucedida
    pub const OK: u8 = 0;
    /// Erro de I/O
    pub const IOERR: u8 = 1;
    /// Operação não suportada
    pub const UNSUPP: u8 = 2;
}

/// Offsets dos registradores VirtIO Legacy (BAR0)
pub mod offsets {
    /// Features suportadas pelo dispositivo (32 bits)
    pub const DEVICE_FEATURES: u64 = 0x00;
    /// Features habilitadas pelo driver (32 bits)
    pub const DRIVER_FEATURES: u64 = 0x04;
    /// Endereço físico da queue (PFN)
    pub const QUEUE_ADDRESS: u64 = 0x08;
    /// Tamanho da queue (16 bits)
    pub const QUEUE_SIZE: u64 = 0x0C;
    /// Seletor de queue (16 bits)
    pub const QUEUE_SELECT: u64 = 0x0E;
    /// Notificação de queue (16 bits)
    pub const QUEUE_NOTIFY: u64 = 0x10;
    /// Status do dispositivo (8 bits)
    pub const DEVICE_STATUS: u64 = 0x12;
    /// Status de interrupção (8 bits)
    pub const ISR_STATUS: u64 = 0x13;
    /// Capacidade do bloco (offset 0x14+, 8 bytes)
    pub const BLK_CAPACITY: u64 = 0x14;
}

/// Status do dispositivo VirtIO
pub mod status {
    /// Reset do dispositivo
    pub const RESET: u8 = 0;
    /// Driver reconheceu o dispositivo
    pub const ACKNOWLEDGE: u8 = 1;
    /// Driver conhece como usar o dispositivo
    pub const DRIVER: u8 = 2;
    /// Driver está pronto para operar
    pub const DRIVER_OK: u8 = 4;
    /// Negociação de features concluída
    pub const FEATURES_OK: u8 = 8;
    /// Dispositivo encontrou erro irrecuperável
    pub const FAILED: u8 = 128;
}

/// Features do dispositivo VirtIO Block
pub mod features {
    /// Suporte a barrier/flush
    pub const BLK_F_BARRIER: u32 = 1 << 0;
    /// Tamanho máximo de segmento disponível
    pub const BLK_F_SIZE_MAX: u32 = 1 << 1;
    /// Número máximo de segmentos
    pub const BLK_F_SEG_MAX: u32 = 1 << 2;
    /// Geometria do disco disponível
    pub const BLK_F_GEOMETRY: u32 = 1 << 4;
    /// Disco é somente leitura
    pub const BLK_F_RO: u32 = 1 << 5;
    /// Tamanho do bloco disponível
    pub const BLK_F_BLK_SIZE: u32 = 1 << 6;
    /// Cache flush suportado
    pub const BLK_F_FLUSH: u32 = 1 << 9;
    /// Suporte a discard
    pub const BLK_F_DISCARD: u32 = 1 << 13;
    /// Suporte a write zeroes
    pub const BLK_F_WRITE_ZEROES: u32 = 1 << 14;
}
