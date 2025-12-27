//! # Kernel Driver Layer - Minimal Assembly Implementation
//!
//! O módulo `drivers` contém os drivers de hardware essenciais do kernel,
//! todos implementados em assembly puro para garantir zero SSE/AVX.
//!
//! ## Drivers Implementados
//!
//! | Driver   | Arquivo      | Status |
//! |----------|--------------|--------|
//! | Serial   | `serial.rs`  | ✅ 100% ASM - Logging de kernel |
//! | PIC      | `pic.rs`     | ✅ 100% ASM - Controlador de interrupções |
//! | Timer    | `timer.rs`   | ✅ 100% ASM I/O - PIT 8254 + scheduler tick |
//! | Video    | `video/`     | Minimal - Apenas framebuffer básico |
//!
//! ## Arquitetura
//!
//! O kernel mantém APENAS drivers essenciais para boot e diagnóstico.
//! Drivers complexos (GPU, rede, som, USB) são carregados como módulos
//! ou rodam em userspace via PID1.
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │              Userspace (PID1+)              │
//! │  - Terminal gráfico                         │
//! │  - Drivers de GPU/Rede/Som via módulos      │
//! └─────────────────────────────────────────────┘
//!                      ↑
//!                   syscalls
//!                      ↑
//! ┌─────────────────────────────────────────────┐
//! │              Kernel (Forge)                 │
//! │  - Serial: logs de diagnóstico              │
//! │  - PIC: interrupções legacy                 │
//! │  - Timer: heartbeat do scheduler            │
//! │  - Video: framebuffer mínimo para panic     │
//! └─────────────────────────────────────────────┘
//! ```

pub mod pic; // 8259 PIC - Interrupções legacy
pub mod serial; // UART 16550 - Logs (100% ASM)
pub mod timer; // PIT 8254 - Timer do sistema
pub mod video; // Framebuffer minimal

#[cfg(feature = "self_test")]
pub mod test;
