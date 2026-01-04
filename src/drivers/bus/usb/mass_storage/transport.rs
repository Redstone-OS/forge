//! # USB Mass Storage BBB Transport
//!
//! Implementa o protocolo Bulk-Only Transport (BBB) para USB Mass Storage.
//!
//! ## Fluxo:
//! 1. Host envia CBW (Command Block Wrapper)
//! 2. Host/Device transfere dados (se necessário)
//! 3. Device envia CSW (Command Status Wrapper)

// =============================================================================
// CONSTANTES
// =============================================================================

/// Signature do CBW.
pub const CBW_SIGNATURE: u32 = 0x43425355; // "USBC"

/// Signature do CSW.
pub const CSW_SIGNATURE: u32 = 0x53425355; // "USBS"

/// Tamanho do CBW.
pub const CBW_SIZE: usize = 31;

/// Tamanho do CSW.
pub const CSW_SIZE: usize = 13;

// =============================================================================
// COMMAND BLOCK WRAPPER (CBW)
// =============================================================================

/// CBW - enviado pelo host antes de cada comando SCSI.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct Cbw {
    /// Signature: 0x43425355 ("USBC").
    pub signature: u32,

    /// Tag para identificar a transação.
    pub tag: u32,

    /// Número de bytes de dados a transferir.
    pub data_transfer_length: u32,

    /// Flags (bit 7 = direção: 1=IN, 0=OUT).
    pub flags: u8,

    /// LUN (bits 3:0).
    pub lun: u8,

    /// Tamanho do CDB (1-16).
    pub cb_length: u8,

    /// Command Block (CDB).
    pub cb: [u8; 16],
}

impl Cbw {
    /// Cria um novo CBW.
    pub fn new(tag: u32, data_length: u32, direction_in: bool, lun: u8, cdb: &[u8]) -> Self {
        let mut cbw = Self {
            signature: CBW_SIGNATURE,
            tag,
            data_transfer_length: data_length,
            flags: if direction_in { 0x80 } else { 0x00 },
            lun: lun & 0x0F,
            cb_length: cdb.len().min(16) as u8,
            cb: [0; 16],
        };

        // Copia CDB
        let len = cdb.len().min(16);
        cbw.cb[..len].copy_from_slice(&cdb[..len]);

        cbw
    }

    /// Converte para bytes.
    pub fn as_bytes(&self) -> [u8; CBW_SIZE] {
        unsafe { core::mem::transmute_copy(self) }
    }
}

// =============================================================================
// COMMAND STATUS WRAPPER (CSW)
// =============================================================================

/// CSW - recebido do device após cada comando.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct Csw {
    /// Signature: 0x53425355 ("USBS").
    pub signature: u32,

    /// Tag (deve corresponder ao CBW).
    pub tag: u32,

    /// Residue: bytes não transferidos.
    pub data_residue: u32,

    /// Status do comando.
    pub status: u8,
}

impl Csw {
    /// Valida o CSW.
    pub fn is_valid(&self, expected_tag: u32) -> bool {
        self.signature == CSW_SIGNATURE && self.tag == expected_tag
    }

    /// Verifica se comando foi bem-sucedido.
    pub fn is_success(&self) -> bool {
        self.status == CSW_STATUS_PASSED
    }

    /// Cria a partir de bytes.
    pub fn from_bytes(bytes: &[u8; CSW_SIZE]) -> Self {
        unsafe { core::mem::transmute_copy(bytes) }
    }
}

// CSW Status codes
pub const CSW_STATUS_PASSED: u8 = 0x00;
pub const CSW_STATUS_FAILED: u8 = 0x01;
pub const CSW_STATUS_PHASE_ERROR: u8 = 0x02;

// =============================================================================
// FUNÇÕES DE TRANSPORTE
// =============================================================================

/// Executa uma transação BBB completa.
///
/// ## STUB:
/// Não executa realmente.
pub fn execute_command(
    _usb_address: u8,
    _bulk_out: u8,
    _bulk_in: u8,
    _cdb: &[u8],
    _data: Option<&mut [u8]>,
    _direction_in: bool,
) -> Result<u32, BbbError> {
    crate::kwarn!("(BBB) execute_command() stub");

    // TODO:
    // 1. Construir CBW
    // 2. Enviar CBW via Bulk OUT
    // 3. Transferir dados (se houver)
    // 4. Receber CSW via Bulk IN
    // 5. Validar CSW

    Err(BbbError::NotImplemented)
}

/// Erros do transporte BBB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BbbError {
    /// Erro de USB.
    UsbError,
    /// CSW inválido.
    InvalidCsw,
    /// Comando falhou.
    CommandFailed,
    /// Erro de fase.
    PhaseError,
    /// Não implementado.
    NotImplemented,
}
