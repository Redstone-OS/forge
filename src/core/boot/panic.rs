//! Panic handler do kernel

use core::panic::PanicInfo;

/// Handler de panic do kernel
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Desabilitar interrupções imediatamente
    crate::arch::Cpu::disable_interrupts();
    
    crate::kerror!("=== KERNEL PANIC ===");
    
    if let Some(location) = info.location() {
        crate::kerror!("File:", location.file().as_ptr() as u64);
        crate::kerror!("Line:", location.line() as u64);
    }
    
    if let Some(msg) = info.message() {
        // Não podemos formatar facilmente, apenas indicar que há mensagem
        crate::kerror!("Panic message presente");
    }
    
    // Halt loop
    loop {
        crate::arch::Cpu::halt();
    }
}
