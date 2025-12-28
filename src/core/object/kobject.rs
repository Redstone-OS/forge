/// Arquivo: core/object/kobject.rs
///
/// Propósito: Definição base para Objetos do Kernel (Kernel Objects).
/// Todo recurso gerenciável via Handle (Processo, Thread, VMO, Canal, etc.)
/// deve implementar o trait `KObject`.
///
/// Detalhes de Implementação:
/// - IDs únicos globais (KOID).
/// - Polimorfismo via Trait Objects (dyn KObject).
/// - Integração com RefCount (geralmente via Arc<KObject> ou similar customizado).

//! Kernel Object Base

use core::sync::atomic::{AtomicU64, Ordering};

/// Kernel Object ID
pub type Koid = u64;

/// Gerador de KOIDs
static KOID_GENERATOR: AtomicU64 = AtomicU64::new(1);

/// Gera um novo KOID único
pub fn generate_koid() -> Koid {
    KOID_GENERATOR.fetch_add(1, Ordering::Relaxed)
}

/// Trait base que todos os objetos do kernel gerenciáveis devem implementar.
pub trait KObject: Send + Sync {
    /// Retorna o ID único do objeto.
    fn koid(&self) -> Koid;

    /// Retorna o nome do tipo do objeto (para debug/diagnóstico).
    fn type_name(&self) -> &'static str;

    /// Chamado quando a última referência (handle ou pointer) ao objeto é solta.
    /// É o destrutor lógico.
    fn on_final_release(&self) {
        // Default: nada (Rust Drop cuida da memória)
    }
}
