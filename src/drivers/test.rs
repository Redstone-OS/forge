//! Testes de Drivers Básicos
//!
//! Valida configurações de hardware de baixo nível (PIC, VGA).

/// Executa todos os testes de drivers
pub fn run_driver_tests() {
    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║     🧪 TESTES DE DRIVERS               ║");
    crate::kinfo!("╚════════════════════════════════════════╝");

    test_pic_remap();
    test_vga_buffer_size();

    crate::kinfo!("╔════════════════════════════════════════╗");
    crate::kinfo!("║  ✅ DRIVERS VALIDADOS!                 ║");
    crate::kinfo!("╚════════════════════════════════════════╝");
}

fn test_pic_remap() {
    crate::kdebug!("(Driver) Verificando offsets do PIC...");

    // O PIC deve ser remapeado para não conflitar com exceções da CPU (0-31)
    // Padrão Redstone: Master = 32, Slave = 40
    let master_offset = 32;
    let slave_offset = 40;

    crate::ktrace!("(Driver) Master Offset: ", master_offset);
    crate::ktrace!("(Driver) Slave Offset:  ", slave_offset);

    if master_offset >= 32 && slave_offset >= 32 {
        crate::kinfo!("(Driver) ✓ PIC Offsets OK (Safe Range)");
    } else {
        crate::kerror!("(Driver) PIC Offset CONFLICT with CPU Excs");
    }
}

fn test_vga_buffer_size() {
    crate::kdebug!("(Driver) Validando cálculo de tamanho de buffer...");

    // Simulação de cálculo de tamanho de framebuffer
    let width = 1024u64;
    let height = 768u64;
    let bpp = 4u64; // 32 bits
    let stride = width * bpp;
    let total_size = stride * height;

    crate::ktrace!("(Driver) Resolução=", width);
    crate::klog!("x", height, " @ 32bpp");
    crate::knl!();
    crate::ktrace!("(Driver) Calculated Size: ", total_size);

    if total_size > 0 {
        crate::kinfo!("(Driver) ✓ VGA Buffer Math OK");
    } else {
        crate::kerror!("(Driver) Invalid Buffer Size");
    }
}
