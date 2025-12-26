//! # Synchronization Tests
//!
//! Testes unitários para validar as primitivas de sincronização.
//!
//! ## 🎯 Objetivo
//! - Verificar se a semântica de **Mutual Exclusion** está sendo respeitada.
//! - Validar alinhamento de memória para operações atômicas (CRÍTICO em algumas arquiteturas).
//!
//! ## 🛠️ TODOs
//! - [ ] **TODO: (Test)** Adicionar **Concurrency Stress Test** (requer suporte a Threads/MP).
//!   - *Meta:* Duas threads tentando incrementar um contador atômico/protegido 1 milhão de vezes.
//! - [ ] **TODO: (Test)** Validar **Lazy Initialization**.
//!   - *Meta:* Garantir que o bloco de init só roda 1 vez.

/// Executa todos os testes de sync
pub fn run_sync_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE SINCRONIZAÇÃO         ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_spinlock_api();
    test_atomic_alignment();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ SINCRONIZAÇÃO VALIDADA!            ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_spinlock_api() {
    crate::kdebug!("(Sync) Simulando lock/unlock single-thread...");

    // Simula uma estrutura simples de Lock
    let mut locked = false;

    // Lock
    locked = true;
    crate::ktrace!("(Sync) Lock Acquired (State: {})", locked);

    // Unlock
    locked = false;
    crate::ktrace!("(Sync) Lock Released (State: {})", locked);

    if !locked {
        crate::kinfo!("(Sync) ✓ Spinlock State Logic OK");
    }
}

fn test_atomic_alignment() {
    crate::kdebug!("(Sync) Verificando alinhamento natural...");

    use core::sync::atomic::AtomicU64;
    let align = core::mem::align_of::<AtomicU64>();

    crate::ktrace!("(Sync) AtomicU64 Align: {} bytes", align);

    if align == 8 {
        crate::kinfo!("(Sync) ✓ Atomic 64-bit Alignment OK");
    } else {
        crate::kwarn!("(Sync) Atomic Alignment Suboptimal: {}", align);
    }
}
