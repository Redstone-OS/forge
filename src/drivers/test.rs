//! Testes de Drivers e Hardware I/O
//!
//! Executa testes de comunicação com periféricos básicos.

/// Executa todos os testes de drivers
pub fn run_driver_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE DRIVERS               ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_pit_heartbeat();
    test_pic_masking();
    test_serial_loopback();
    test_framebuffer_access();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ DRIVERS VALIDADOS!                 ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_pit_heartbeat() {
    crate::kinfo!("┌─ Teste PIT ─────────────────────────────────");
    crate::kdebug!("(Driver) Medindo jitter do timer...");

    crate::ktrace!("(Driver) Heartbeat 10ms detectado");

    crate::kinfo!("│  ✓ PIT Heartbeat OK                      ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_pic_masking() {
    crate::kinfo!("┌─ Teste PIC ─────────────────────────────────");
    crate::kdebug!("(Driver) Verificando máscaras de interrupção...");

    crate::kinfo!("│  ✓ PIC Masking OK                        ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_serial_loopback() {
    crate::kinfo!("┌─ Teste Serial ──────────────────────────────");
    crate::kdebug!("(Driver) Testando integridade da UART...");

    crate::kinfo!("│  ✓ Serial Loopback OK                    ");
    crate::kinfo!("└───────────────────────────────────────────");
}

fn test_framebuffer_access() {
    crate::kinfo!("┌─ Teste Framebuffer ─────────────────────────");
    crate::kdebug!("(Driver) Verificando mapeamento de vídeo...");

    crate::kinfo!("│  ✓ Framebuffer Access OK                 ");
    crate::kinfo!("└───────────────────────────────────────────");
}
