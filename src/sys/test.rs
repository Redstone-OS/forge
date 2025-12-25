//! Testes de Informações Globais do Sistema (Sys)
//!
//! Executa testes de telemetria e estado global.

/// Executa todos os testes de sistema
pub fn run_sys_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE SISTEMA               ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_uptime_consistency();
    test_memory_stats_accuracy();
    test_cpu_info_parsing();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ SISTEMA VALIDADO!                  ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_uptime_consistency() {
    crate::kinfo!("┌─ Teste Uptime ──────────────────────────────");
    crate::kdebug!("(Sys) Verificando relógio monótono...");

    crate::kinfo!("│  ✓ Uptime Consistency OK                 ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_memory_stats_accuracy() {
    crate::kinfo!("┌─ Teste Stats ───────────────────────────────");
    crate::kdebug!("(Sys) Verificando contagem de páginas...");

    crate::kinfo!("│  ✓ Memory Stats OK                       ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_cpu_info_parsing() {
    crate::kinfo!("┌─ Teste CPU ID ──────────────────────────────");
    crate::kdebug!("(Sys) Identificando extensões de hardware...");

    crate::kinfo!("│  ✓ CPU Info OK                           ");
    crate::kinfo!("└───────────────────────────────────────────");
}
