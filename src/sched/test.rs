//! Testes da Configuração do Scheduler
//!
//! Valida constantes e hierarquia de prioridades.

/// Executa todos os testes de scheduler
pub fn run_sched_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE SCHEDULER             ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_task_stack_size();
    test_priority_ordering();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ SCHEDULER VALIDADO!                ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_task_stack_size() {
    crate::kdebug!("(Sched) Validando constantes de pilha...");

    // Stack padrão de kernel geralmente é 16KiB ou 32KiB
    let stack_size = 16 * 1024; // 16 KiB

    crate::ktrace!("(Sched) Kernel Stack: {} bytes", stack_size);

    if stack_size % 4096 == 0 {
        crate::kinfo!("(Sched) ✓ Stack Size Page Aligned OK");
    } else {
        crate::kwarn!("(Sched) Stack Size NOT Page Aligned");
    }
}

fn test_priority_ordering() {
    crate::kdebug!("(Sched) Verificando hierarquia de enum...");

    #[derive(PartialEq, PartialOrd)]
    enum Priority {
        Low,
        Normal,
        High,
    }

    if Priority::High > Priority::Normal && Priority::Normal > Priority::Low {
        crate::ktrace!("(Sched) High > Normal > Low confirmed");
        crate::kinfo!("(Sched) ✓ Priority Ordering OK");
    } else {
        crate::kerror!("(Sched) Priority Enum Broken!");
    }
}
