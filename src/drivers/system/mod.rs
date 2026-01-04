//! # System Drivers Layer
//!
//! Este módulo contém os drivers de componentes de sistema do RedstoneOS.
//! Componentes críticos para o funcionamento do kernel.
//!
//! ## Arquitetura:
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           Kernel Core                   │
//! ├─────────────────────────────────────────┤
//! │          System Subsystem               │  (este módulo)
//! ├──────────┬────────┬─────────┬───────────┤
//! │  Timer   │ IntCtl │ PwrCtl  │   DMA     │
//! ├──────────┴────────┴─────────┴───────────┤
//! │              drivers/base               │  (RDS Core)
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Componentes:
//!
//! | Componente | Status | Descrição                         |
//! |------------|--------|-----------------------------------|
//! | `timer`    | Func   | PIT, HPET, TSC                    |
//! | `int_ctrl` | Stub   | PIC, APIC, IO-APIC                |
//! | `pwr_ctrl` | Stub   | Shutdown, Reboot, Sleep           |
//! | `dma`      | Stub   | Legacy DMA (8237)                 |
//! | `speaker`  | Stub   | PC Speaker (beep)                 |
//!
//! ## Prioridade de Inicialização:
//! 1. Interrupt Controllers (crítico para IRQs)
//! 2. Timers (crítico para scheduler)
//! 3. DMA (necessário para alguns drivers)
//! 4. Power Control
//! 5. Misc (speaker)

pub mod dma;
pub mod int_ctrl;
pub mod pwr_ctrl;
pub mod speaker;
pub mod timer;
pub mod traits;

// Re-exports
pub use traits::*;

use crate::sync::Spinlock;

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

/// Flag de inicialização.
static INITIALIZED: Spinlock<bool> = Spinlock::new(false);

// =============================================================================
// INICIALIZAÇÃO
// =============================================================================

/// Inicializa o subsistema de sistema.
///
/// ATENÇÃO: Esta função é chamada muito cedo no boot.
/// Os componentes aqui são críticos e devem funcionar corretamente.
pub fn init() {
    crate::kinfo!("(System) Inicializando componentes de sistema...");

    // Fase 1: Controladores de Interrupção (CRÍTICO)
    // Necessário para qualquer IRQ funcionar
    int_ctrl::init();

    // Fase 2: Timers (CRÍTICO)
    // Necessário para scheduler e delays
    timer::init();

    // Fase 3: DMA (legacy)
    // Necessário para floppy, sound blaster, etc
    dma::init();

    // Fase 4: Power Control
    // Shutdown/reboot
    pwr_ctrl::init();

    // Fase 5: Misc
    speaker::init();

    *INITIALIZED.lock() = true;
    crate::kinfo!("(System) Componentes de sistema prontos");
}

/// Desliga o subsistema.
pub fn shutdown() {
    crate::kinfo!("(System) Shutdown de componentes...");
    // Componentes de sistema geralmente não precisam de shutdown especial
}

/// Verifica se o subsistema está inicializado.
pub fn is_initialized() -> bool {
    *INITIALIZED.lock()
}

// =============================================================================
// FUNÇÕES DE CONVENIÊNCIA
// =============================================================================

/// Reinicia o sistema.
pub fn reboot() {
    pwr_ctrl::reboot();
}

/// Desliga o sistema.
pub fn power_off() {
    pwr_ctrl::shutdown();
}

/// Retorna ticks do sistema.
pub fn ticks() -> u64 {
    timer::ticks()
}

/// Retorna frequência do timer em Hz.
pub fn timer_frequency() -> u64 {
    timer::frequency()
}

/// Delay em milissegundos.
pub fn delay_ms(ms: u64) {
    timer::delay_ms(ms);
}

/// Delay em microsegundos (busy-wait).
pub fn delay_us(us: u64) {
    timer::delay_us(us);
}

/// Toca beep no speaker.
pub fn beep(frequency_hz: u32, duration_ms: u64) {
    speaker::beep(frequency_hz, duration_ms);
}
