//! Panic Handler.
//!
//! O "Airbag" do sistema. Quando o Rust detecta um estado irrecuperável,
//! esta função é chamada.
//!
//! # Comportamento
//! 1. Desabilita interrupções (evita loop de panics).
//! 2. Loga o erro na Serial (para o desenvolvedor).
//! 3. Trava a CPU (hlt loop).

use crate::arch::platform::Cpu;
use crate::arch::traits::CpuOps;
use core::panic::PanicInfo; // Uso agnóstico da plataforma atual

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 1. Segurança imediata: parar interrupções
    Cpu::disable_interrupts();

    // 2. Log estruturado (Serial é o mais confiável aqui)
    // Usamos kerror! diretamente. Se o logger falhar, estamos perdidos de qualquer forma.
    crate::kerror!("================ KERNEL PANIC ================");

    if let Some(location) = info.location() {
        crate::kerror!("Location: {}:{}", location.file(), location.line());
    } else {
        crate::kerror!("Location: Unknown");
    }

    crate::kerror!(
        "Reason:   {}",
        info.message().unwrap_or(&format_args!("Unknown error"))
    );
    crate::kerror!("==============================================");

    // 3. Morrer com dignidade
    Cpu::hang();
}
