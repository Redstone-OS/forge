//! Panic Handler - Tratamento de pânicos do kernel
//!
//! Implementação do panic handler obrigatório para kernels no_std.

use core::panic::PanicInfo;

/// Panic handler do kernel
///
/// Esta função é chamada quando ocorre um panic no kernel.
/// Por enquanto, apenas para no loop infinito.
///
/// TODO(prioridade=alta, versão=v1.0): Implementar panic handler completo
/// - Imprimir informações de debug via serial
/// - Mostrar stack trace
/// - Dump de registradores
/// - Red screen of death (RSOD)
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // TODO: Imprimir informações via serial
    // if let Some(location) = info.location() {
    //     serial_println!("PANIC at {}:{}", location.file(), location.line());
    // }
    // if let Some(message) = info.message() {
    //     serial_println!("Message: {}", message);
    // }

    // Por enquanto, apenas loop infinito
    loop {
        // Halt CPU para economizar energia
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// TODO(prioridade=média, versão=v1.0): Implementar funções auxiliares
// - print_panic_info() - Formatar e imprimir informações
// - dump_registers() - Dump de registradores da CPU
// - print_stack_trace() - Stack trace
// - halt_all_cpus() - Parar todas as CPUs em SMP
