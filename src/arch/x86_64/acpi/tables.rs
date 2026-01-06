//! # ACPI Tables
//!
//! Estruturas das tabelas ACPI (RSDP, SDT Header, etc.)

/// Root System Description Pointer (RSDP)
///
/// Estrutura fornecida pelo firmware para localizar as tabelas ACPI.
/// Existem duas versões:
/// - v1 (ACPI 1.0): Apenas RSDT (32-bit)
/// - v2+ (ACPI 2.0+): Adiciona XSDT (64-bit)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Rsdp {
    /// Assinatura "RSD PTR " (8 bytes com espaço)
    pub signature: [u8; 8],
    /// Checksum do primeiro bloco (20 bytes)
    pub checksum: u8,
    /// OEM ID
    pub oem_id: [u8; 6],
    /// Revisão (0 = ACPI 1.0, 2 = ACPI 2.0+)
    pub revision: u8,
    /// Endereço físico da RSDT (32-bit)
    pub rsdt_address: u32,

    // --- Campos adicionais ACPI 2.0+ ---
    /// Tamanho total da estrutura
    pub length: u32,
    /// Endereço físico da XSDT (64-bit)
    pub xsdt_address: u64,
    /// Checksum estendido
    pub extended_checksum: u8,
    /// Reservado
    pub reserved: [u8; 3],
}

impl Rsdp {
    /// Valida a assinatura e checksum do RSDP
    pub fn validate(&self) -> bool {
        // Verificar assinatura
        if &self.signature != b"RSD PTR " {
            return false;
        }

        // Verificar checksum do primeiro bloco (20 bytes)
        let bytes = unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, 20) };
        let sum: u8 = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

        sum == 0
    }
}

/// System Description Table Header
///
/// Header comum a todas as tabelas ACPI (RSDT, XSDT, MADT, FADT, etc.)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct SdtHeader {
    /// Assinatura de 4 caracteres (ex: "APIC", "FACP")
    pub signature: [u8; 4],
    /// Tamanho total da tabela incluindo header
    pub length: u32,
    /// Revisão da tabela
    pub revision: u8,
    /// Checksum (toda a tabela deve somar 0)
    pub checksum: u8,
    /// OEM ID
    pub oem_id: [u8; 6],
    /// OEM Table ID
    pub oem_table_id: [u8; 8],
    /// OEM Revision
    pub oem_revision: u32,
    /// Creator ID
    pub creator_id: u32,
    /// Creator Revision
    pub creator_revision: u32,
}

impl SdtHeader {
    /// Valida o checksum da tabela
    pub unsafe fn validate(&self) -> bool {
        let bytes =
            core::slice::from_raw_parts(self as *const _ as *const u8, self.length as usize);
        let sum: u8 = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        sum == 0
    }

    /// Retorna a assinatura como string
    pub fn signature_str(&self) -> &str {
        core::str::from_utf8(&self.signature).unwrap_or("????")
    }
}
