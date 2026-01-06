//! # Page Fault Handler
//!
//! Tratamento unificado de page faults para lazy allocation, COW, e swap.
//!
//! ## Fluxo
//!
//! ```text
//! Page Fault
//!     │
//!     ├─► É endereço válido na VMA?
//!     │       │
//!     │       ├─► Sim: Lazy alloc / COW / Swap-in
//!     │       │
//!     │       └─► Não: Segmentation Fault
//!     │
//!     └─► Retorna resultado para handler de exceção
//! ```

/// Informações sobre um page fault
#[derive(Debug, Clone)]
pub struct PageFaultInfo {
    /// Endereço que causou a falta
    pub address: u64,
    /// Instrução que causou a falta
    pub instruction_pointer: u64,
    /// Falta causada por escrita?
    pub is_write: bool,
    /// Falta causada por execução?
    pub is_execute: bool,
    /// Falta em modo usuário?
    pub is_user: bool,
    /// Página estava presente?
    pub page_present: bool,
}

impl PageFaultInfo {
    /// Cria PageFaultInfo a partir do error_code do x86_64
    pub fn from_error_code(address: u64, instruction_pointer: u64, error_code: u64) -> Self {
        // Bits do error_code:
        // 0: P (presente)
        // 1: W/R (write = 1)
        // 2: U/S (user = 1)
        // 4: I/D (instruction fetch = 1)
        Self {
            address,
            instruction_pointer,
            is_write: (error_code & 0x2) != 0,
            is_execute: (error_code & 0x10) != 0,
            is_user: (error_code & 0x4) != 0,
            page_present: (error_code & 0x1) != 0,
        }
    }
}

/// Resultado do tratamento de page fault
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultResult {
    /// Falta resolvida com sucesso (lazy alloc, COW, swap-in)
    Success,
    /// Endereço inválido (não mapeado em nenhuma VMA)
    InvalidAddress,
    /// Violação de proteção (ex: escrita em página read-only não-COW)
    ProtectionViolation,
    /// Fora de memória
    OutOfMemory,
    /// Swap-in falhou
    SwapError,
    /// Erro interno
    InternalError,
}

/// Handler principal de page faults
///
/// Chamado pelo handler de exceção #PF. Retorna Success se a falta
/// foi resolvida e a instrução pode ser re-executada.
pub fn handle_page_fault(info: PageFaultInfo) -> FaultResult {
    // TODO: Implementar tratamento completo
    //
    // 1. Obter AddressSpace da task atual
    // 2. Lookup VMA para info.address
    // 3. Se não há VMA, retorna InvalidAddress
    // 4. Verifica proteção da VMA vs operação
    // 5. Se página não presente:
    //    a. Se VMA é lazy, aloca página zerada
    //    b. Se VMA tem swap entry, faz swap-in
    //    c. Se VMA é file-backed, carrega do arquivo
    // 6. Se página presente mas escrita em COW:
    //    a. Faz cópia da página
    //    b. Atualiza mapeamento
    // 7. Flush TLB para o endereço
    // 8. Retorna Success

    // Por enquanto: implementação stub
    // Apenas retorna erro para que o handler de exceção trate

    crate::ktrace!("(RMM/Fault) addr=", info.address);
    crate::ktrace!("(RMM/Fault) write=", info.is_write as u64);
    crate::ktrace!("(RMM/Fault) user=", info.is_user as u64);

    // Stub: sempre falha (será implementado depois)
    FaultResult::InvalidAddress
}
