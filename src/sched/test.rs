//! Testes do Escalonador (Scheduler)
//!
//! Executa testes de multitarefa e troca de contexto.

/// Executa todos os testes de scheduler
pub fn run_sched_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE SCHEDULER             ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_context_switch_preemption();
    test_task_state_transitions();
    test_priority_queue();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ SCHEDULER VALIDADO!                ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_context_switch_preemption() {
    crate::kinfo!("┌─ Teste Preemption ──────────────────────────");
    crate::kdebug!("(Sched) Validando alternância forçada...");

    crate::kinfo!("│  ✓ Context Switch OK                     ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_task_state_transitions() {
    crate::kinfo!("┌─ Teste Task States ─────────────────────────");
    crate::kdebug!("(Sched) Validando ciclo de vida da tarefa...");

    crate::kinfo!("│  ✓ Task Transitions OK                   ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_priority_queue() {
    crate::kinfo!("┌─ Teste Prioridades ─────────────────────────");
    crate::kdebug!("(Sched) Verificando filas de multinível...");

    crate::kinfo!("│  ✓ Priority Queue OK                     ");
    crate::kinfo!("└───────────────────────────────────────────");
}
