//! # System Traits and Types
//!
//! Define interfaces para componentes de sistema.

use crate::core::debug::klog::SerialPrint;
use alloc::sync::Arc;

// =============================================================================
// TIMER TRAITS
// =============================================================================

/// Fonte de tempo do sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerSource {
    /// Programmable Interval Timer (8254).
    Pit,
    /// High Precision Event Timer.
    Hpet,
    /// Timestamp Counter (CPU cycles).
    Tsc,
    /// ACPI PM Timer.
    AcpiPm,
    /// Local APIC Timer.
    LocalApic,
}

impl SerialPrint for TimerSource {
    fn serial_print(&self) {
        match self {
            Self::Pit => "PIT",
            Self::Hpet => "HPET",
            Self::Tsc => "TSC",
            Self::AcpiPm => "ACPI PM",
            Self::LocalApic => "Local APIC",
        }
        .serial_print();
    }
}

/// Capacidades de um timer.
#[derive(Debug, Clone, Copy, Default)]
pub struct TimerCapabilities {
    /// Frequência em Hz (0 = variável).
    pub frequency_hz: u64,
    /// Resolução em nanosegundos.
    pub resolution_ns: u64,
    /// Suporta one-shot mode.
    pub one_shot: bool,
    /// Suporta periodic mode.
    pub periodic: bool,
    /// É monotônico (nunca decrementa).
    pub monotonic: bool,
    /// Contador de 64 bits.
    pub bits_64: bool,
}

/// Interface para dispositivos de timer.
pub trait TimerDevice: Send + Sync {
    /// Nome do timer.
    fn name(&self) -> &str;

    /// Tipo de fonte.
    fn source(&self) -> TimerSource;

    /// Capacidades.
    fn capabilities(&self) -> TimerCapabilities;

    /// Lê valor atual do contador.
    fn read(&self) -> u64;

    /// Configura timer em modo periódico.
    fn set_periodic(&self, frequency_hz: u32) -> bool {
        false
    }

    /// Configura timer em modo one-shot.
    fn set_oneshot(&self, ticks: u64) -> bool {
        false
    }

    /// Para o timer.
    fn stop(&self) {}

    /// Habilita IRQ do timer.
    fn enable_irq(&self) {}

    /// Desabilita IRQ do timer.
    fn disable_irq(&self) {}
}

pub type TimerDeviceRef = Arc<dyn TimerDevice>;

// =============================================================================
// INTERRUPT CONTROLLER TRAITS
// =============================================================================

/// Tipo de controlador de interrupção.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptControllerType {
    /// Programmable Interrupt Controller (8259).
    Pic,
    /// Local Advanced PIC.
    LocalApic,
    /// I/O Advanced PIC.
    IoApic,
    /// MSI (Message Signaled Interrupts).
    Msi,
    /// MSI-X.
    MsiX,
}

impl SerialPrint for InterruptControllerType {
    fn serial_print(&self) {
        match self {
            Self::Pic => "PIC",
            Self::LocalApic => "Local APIC",
            Self::IoApic => "I/O APIC",
            Self::Msi => "MSI",
            Self::MsiX => "MSI-X",
        }
        .serial_print();
    }
}

/// Interface para controladores de interrupção.
pub trait InterruptController: Send + Sync {
    /// Nome do controlador.
    fn name(&self) -> &str;

    /// Tipo.
    fn controller_type(&self) -> InterruptControllerType;

    /// Habilita uma linha de IRQ.
    fn enable_irq(&self, irq: u8);

    /// Desabilita uma linha de IRQ.
    fn disable_irq(&self, irq: u8);

    /// Envia EOI (End of Interrupt).
    fn send_eoi(&self, irq: u8);

    /// Mascara todas as IRQs.
    fn mask_all(&self);

    /// Desmascara todas as IRQs.
    fn unmask_all(&self);

    /// Lê ISR (In-Service Register) - qual IRQ está sendo processada.
    fn read_isr(&self) -> u16 {
        0
    }

    /// Lê IRR (Interrupt Request Register) - quais IRQs estão pendentes.
    fn read_irr(&self) -> u16 {
        0
    }
}

pub type InterruptControllerRef = Arc<dyn InterruptController>;

// =============================================================================
// POWER CONTROL
// =============================================================================

/// Estado de energia do sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    /// Sistema ligado e funcionando (S0).
    Running,
    /// Standby (S1).
    Standby,
    /// Suspend to RAM (S3).
    SuspendToRam,
    /// Hibernate (S4).
    Hibernate,
    /// Desligado (S5).
    Off,
}

impl SerialPrint for PowerState {
    fn serial_print(&self) {
        match self {
            Self::Running => "Running",
            Self::Standby => "Standby",
            Self::SuspendToRam => "Suspend to RAM",
            Self::Hibernate => "Hibernate",
            Self::Off => "Off",
        }
        .serial_print();
    }
}

/// Método de reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetMethod {
    /// ACPI reset register.
    Acpi,
    /// Keyboard controller (8042).
    Keyboard,
    /// Triple fault.
    TripleFault,
    /// PCI reset.
    Pci,
}

impl SerialPrint for ResetMethod {
    fn serial_print(&self) {
        match self {
            Self::Acpi => "ACPI",
            Self::Keyboard => "Keyboard",
            Self::TripleFault => "Triple Fault",
            Self::Pci => "PCI",
        }
        .serial_print();
    }
}

// =============================================================================
// DMA TRAITS
// =============================================================================

/// Canal DMA legado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaChannel {
    Channel0 = 0,
    Channel1 = 1,
    Channel2 = 2,
    Channel3 = 3,
    Channel4 = 4, // Cascade (não usar)
    Channel5 = 5,
    Channel6 = 6,
    Channel7 = 7,
}

impl SerialPrint for DmaChannel {
    fn serial_print(&self) {
        crate::drivers::comm::serial::write_str("Channel ");
        (*self as u8).serial_print();
    }
}

/// Modo de transferência DMA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaMode {
    /// Leitura (periférico -> memória).
    Read,
    /// Escrita (memória -> periférico).
    Write,
    /// Verify (sem transferência real).
    Verify,
}

impl SerialPrint for DmaMode {
    fn serial_print(&self) {
        match self {
            Self::Read => "Read",
            Self::Write => "Write",
            Self::Verify => "Verify",
        }
        .serial_print();
    }
}

/// Erro de DMA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaError {
    /// Canal inválido.
    InvalidChannel,
    /// Endereço cruza limite de 64KB.
    BoundaryCross,
    /// Endereço acima de 16MB (ISA DMA limit).
    AddressTooHigh,
    /// Canal ocupado.
    ChannelBusy,
}

impl SerialPrint for DmaError {
    fn serial_print(&self) {
        match self {
            Self::InvalidChannel => "Invalid Channel",
            Self::BoundaryCross => "Boundary Cross",
            Self::AddressTooHigh => "Address Too High",
            Self::ChannelBusy => "Channel Busy",
        }
        .serial_print();
    }
}
