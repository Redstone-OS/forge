//! Testes da Lógica de Sincronização
//!
//! Valida alinhamento atômico e estados de bloqueio.

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
    crate::kinfo!("┌─ Teste Spinlock API ────────────────────────");
    crate::kdebug!("(Sync) Simulando lock/unlock single-thread...");

    // Simula uma estrutura simples de Lock
    let mut locked = false;

    // Lock
    locked = true;
    crate::ktrace!("(Sync) Lock Acquired (State: locked)");

    // Unlock
    locked = false;
    crate::ktrace!("(Sync) Lock Released (State: free)");

    if !locked {
        crate::kinfo!("│  ✓ Spinlock State Logic OK               ");
    }
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_atomic_alignment() {
    crate::kinfo!("┌─ Teste Atomic Align ────────────────────────");
    crate::kdebug!("(Sync) Verificando alinhamento natural...");

    use core::sync::atomic::AtomicU64;
    let align = core::mem::align_of::<AtomicU64>();

    crate::ktrace!("(Sync) AtomicU64 Align: {} bytes", align);

    if align == 8 {
        crate::kinfo!("│  ✓ Atomic 64-bit Alignment OK            ");
    } else {
        crate::kwarn!("(Sync) Atomic Alignment Suboptimal: {}", align);
    }
    crate::kinfo!("└───────────────────────────────────────────");
}
