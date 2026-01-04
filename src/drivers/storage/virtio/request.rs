//! # Estruturas de Request VirtIO Block
//!
//! Definições das estruturas de requisição para o protocolo VirtIO Block.

/// Header do request VirtIO Block (16 bytes)
///
/// ```text
/// Request Layout:
/// ┌─────────────────┬─────────────────┬─────────────────┐
/// │  Header (16B)   │   Data (512B)   │   Status (1B)   │
/// │  type, sector   │   (R/W buffer)  │   (resultado)   │
/// └─────────────────┴─────────────────┴─────────────────┘
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BlkReqHeader {
    /// Tipo de operação (IN=0, OUT=1, FLUSH=4, GET_ID=8)
    pub req_type: u32,
    /// Reservado (deve ser 0)
    pub reserved: u32,
    /// Setor (LBA - Logical Block Address)
    pub sector: u64,
}

impl BlkReqHeader {
    /// Cria um novo header zerado
    pub const fn new() -> Self {
        Self {
            req_type: 0,
            reserved: 0,
            sector: 0,
        }
    }

    /// Cria um header de leitura
    pub fn read(sector: u64) -> Self {
        Self {
            req_type: super::regs::blk_type::IN,
            reserved: 0,
            sector,
        }
    }

    /// Cria um header de escrita
    pub fn write(sector: u64) -> Self {
        Self {
            req_type: super::regs::blk_type::OUT,
            reserved: 0,
            sector,
        }
    }

    /// Cria um header de flush
    pub fn flush() -> Self {
        Self {
            req_type: super::regs::blk_type::FLUSH,
            reserved: 0,
            sector: 0,
        }
    }
}

impl Default for BlkReqHeader {
    fn default() -> Self {
        Self::new()
    }
}
