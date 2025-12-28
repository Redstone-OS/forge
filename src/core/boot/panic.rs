//! Panic Handler

use crate::arch::platform::Cpu;
use crate::arch::traits::CpuOps;
use crate::drivers::serial;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        Cpu::disable_interrupts();
    }

    serial::write("\n\n=== KERNEL PANIC ===\n");

    if let Some(location) = info.location() {
        serial::write(location.file());
        serial::write("\n");
    }

    serial::write("Sistema congelado.\n");

    Cpu::hang();
}
