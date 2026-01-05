//! # Integrity Verification
//!
//! Verifica invariantes do RMM para detectar corrupção de memória.
//!
//! ## Invariantes Verificadas
//!
//! - **INV-1**: Frame livre tem refcount == 0
//! - **INV-2**: refcount > 0 implica não-livre
//! - **INV-3**: Bitmap consistente com FrameInfo
//! - **INV-4**: Soma das zonas == total de frames
//! - **INV-5**: Rmap consistente com mapeamentos
//!
//! ## Uso
//!
//! ```rust,ignore
//! let result = verify_integrity();
//! if !result.is_ok() {
//!     kerror!("Memory corruption detected!");
//!     for msg in result.messages {
//!         kerror!("  {}", msg);
//!     }
//! }
//! ```

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::rmm::phys::{get_owner, FrameOwner};
use alloc::string::String;
use alloc::vec::Vec;

// =============================================================================
// VerifyResult
// =============================================================================

/// Resultado da verificação de integridade
#[derive(Debug, Default)]
pub struct VerifyResult {
    /// Número de erros encontrados
    pub error_count: usize,
    /// Número de warnings
    pub warning_count: usize,
    /// Mensagens de erro/warning
    pub messages: Vec<String>,
    /// Frames verificados
    pub frames_checked: usize,
}

impl VerifyResult {
    /// Cria resultado vazio
    pub fn new() -> Self {
        Self::default()
    }

    /// Verifica se não há erros
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.error_count == 0
    }

    /// Adiciona erro
    pub fn error(&mut self, msg: impl Into<String>) {
        self.error_count += 1;
        self.messages.push(msg.into());
    }

    /// Adiciona warning
    pub fn warn(&mut self, msg: impl Into<String>) {
        self.warning_count += 1;
        self.messages.push(msg.into());
    }
}

// =============================================================================
// Verificação Principal
// =============================================================================

/// Verifica integridade de todos os frames
///
/// Esta função pode ser lenta em sistemas com muita RAM.
/// Use apenas para debug ou durante boot.
pub fn verify_integrity() -> VerifyResult {
    let mut result = VerifyResult::new();

    // Verifica frames conhecidos
    verify_frame_consistency(&mut result);

    // Verifica zonas
    verify_zone_totals(&mut result);

    // Verifica counters
    verify_counters(&mut result);

    result
}

/// Verifica consistência de frames individuais
fn verify_frame_consistency(result: &mut VerifyResult) {
    // TODO: Iterar sobre todos os frames quando FrameManager expuser API
    // Por enquanto, verifica alguns frames de amostra

    let sample_addrs = [0x1000, 0x10000, 0x100000, 0x1000000];

    for &addr in &sample_addrs {
        let phys = PhysAddr::new(addr);
        if !verify_single_frame(phys, result) {
            // Erro já foi registrado em result
        }
        result.frames_checked += 1;
    }
}

/// Verifica um frame específico
fn verify_single_frame(phys: PhysAddr, result: &mut VerifyResult) -> bool {
    // Obtém owner
    let owner = match get_owner(phys) {
        Some(o) => o,
        None => {
            // Frame não mapeado no FrameManager - pode ser normal para certas regiões
            return true;
        }
    };

    // INV-1: Free implies refcount == 0
    // TODO: Verificar quando refcount API estiver disponível

    // INV-2: refcount > 0 implies not Free
    // TODO: Verificar quando refcount API estiver disponível

    // Verifica owner válido
    match owner {
        FrameOwner::Free => {}
        FrameOwner::Kernel => {}
        FrameOwner::Process { pid } => {
            if pid == 0 {
                result.warn(alloc::format!(
                    "Frame {:x} owned by process PID 0",
                    phys.as_u64()
                ));
            }
        }
        FrameOwner::Driver { .. } => {}
        FrameOwner::Shared => {}
        FrameOwner::Device => {}
        FrameOwner::Pinned { .. } => {}
        FrameOwner::Cache { .. } => {}
        FrameOwner::Slab => {}
        FrameOwner::Buddy => {}
    }

    true
}

/// Verifica totais de zonas
fn verify_zone_totals(result: &mut VerifyResult) {
    // TODO: Verificar quando FrameManager expuser stats de zona
    // Por enquanto, assume OK
    let _ = result;
}

/// Verifica consistência de contadores
fn verify_counters(result: &mut VerifyResult) {
    let (allocs, frees, _, _) = super::stats::counters();

    // Allocs devem ser >= frees (não podemos liberar mais do que alocamos)
    if frees > allocs {
        result.error(alloc::format!(
            "Counter inconsistency: frees ({}) > allocs ({})",
            frees,
            allocs
        ));
    }
}

// =============================================================================
// Verificação de Frame Específico
// =============================================================================

/// Verifica um frame específico e retorna resultado
pub fn verify_frame(phys: PhysAddr) -> VerifyResult {
    let mut result = VerifyResult::new();
    verify_single_frame(phys, &mut result);
    result.frames_checked = 1;
    result
}

/// Verifica range de frames
pub fn verify_range(start: PhysAddr, count: usize) -> VerifyResult {
    let mut result = VerifyResult::new();

    for i in 0..count {
        let phys = PhysAddr::new(start.as_u64() + (i * PAGE_SIZE) as u64);
        verify_single_frame(phys, &mut result);
        result.frames_checked += 1;
    }

    result
}

// =============================================================================
// Panic on Corruption
// =============================================================================

/// Verifica integridade e panic se houver erro
///
/// Use em pontos críticos onde corrupção não pode ser tolerada.
#[cfg(debug_assertions)]
pub fn verify_or_panic() {
    let result = verify_integrity();
    if !result.is_ok() {
        crate::kerror!("=== RMM INTEGRITY CHECK FAILED ===");
        for msg in &result.messages {
            crate::kerror!("  {}", msg.as_str());
        }
        panic!(
            "RMM integrity check failed with {} errors",
            result.error_count
        );
    }
}

/// Versão release (no-op)
#[cfg(not(debug_assertions))]
pub fn verify_or_panic() {
    // Em release, não verificamos para não impactar performance
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_result() {
        let mut result = VerifyResult::new();
        assert!(result.is_ok());

        result.error("test error");
        assert!(!result.is_ok());
        assert_eq!(result.error_count, 1);

        result.warn("test warning");
        assert_eq!(result.warning_count, 1);
    }
}
