/// Arquivo: x86_64/acpi/dsdt.rs
///
/// Propósito: Definição da Differentiated System Description Table (DSDT).
/// Esta tabela contém o código AML (ACPI Machine Language) principal que descreve
/// os dispositivos integrados do sistema, métodos de controle de energia e eventos.
///
/// Detalhes de Implementação:
/// - A DSDT é apontada pela FADT (Fixed Pointer).
/// - Diferente de outras tabelas, o corpo da DSDT não é uma lista simples de estruturas,
///   mas sim bytecode AML compilado que requer um interpretador para ser entendido interativamente.
/// - O Kernel geralmente apenas localiza esta tabela para passá-la a um parser AML (se houver)
///   ou para extrair informações básicas se o bytecode for conhecido.

/// Estrutura DSDT (Differentiated System Description Table)

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Dsdt {
    pub signature: [u8; 4], // "DSDT"
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
    // O restante da tabela é o Definition Block contendo AML Bytecode.
    // O tamanho é (length - sizeof(header)).
}

impl Dsdt {
    /// Verifica se a assinatura é válida
    pub fn validate_signature(&self) -> bool {
        &self.signature == b"DSDT"
    }
}
