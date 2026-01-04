//! # Kernel Debug Utilities
//!
//! Ferramentas de depuração de baixo nível.

/// Dispara um breakpoint de software.
///
/// Para o kernel para inspeção via debugger (GDB/QEMU).
#[inline]
pub fn breakpoint() {
    crate::kwarn!("--- KERNEL BREAKPOINT ---");
    crate::arch::Cpu::disable_interrupts();
    loop {
        crate::arch::Cpu::halt();
    }
}

/// Chamado quando uma assertion falha.
#[cold]
pub fn assert_failed(expr: &str, file: &str, line: u32) -> ! {
    crate::kerror!("=== ASSERTION FAILED ===");
    crate::kerror!("Expression:", expr);
    crate::kerror!("File:", file);
    crate::kerror!("Line:", line);
    panic!("Assertion failed at {}:{}", file, line);
}

/// Macro de assertion customizada.
#[macro_export]
macro_rules! kassert {
    ($cond:expr) => {
        if !$cond {
            $crate::core::debug::kdebug::assert_failed(stringify!($cond), file!(), line!());
        }
    };
    ($cond:expr, $msg:expr) => {
        if !$cond {
            $crate::kerror!($msg);
            $crate::core::debug::kdebug::assert_failed(stringify!($cond), file!(), line!());
        }
    };
}
