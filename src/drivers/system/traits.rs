//! # System Traits and Types
//!
//! Define interfaces para componentes de sistema.

use crate::core::debug::klog::SerialPrintTo;
use crate::drivers::comm::serial::SerialPort;
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

impl SerialPrintTo for TimerSource {
    fn serial_print_to(&self, s: &mut SerialPort) {
        match self {
            Self::Pit => s.write_str("PIT"),
            Self::Hpet => s.write_str("HPET"),
            Self::Tsc => s.write_str("TSC"),
            Self::AcpiPm => s.write_str("ACPI PM"),
            Self::LocalApic => s.write_str("Local APIC"),
        }
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
    // TODO: Revisar no futuro
    #[allow(unused)]
    fn set_periodic(&self, frequency_hz: u32) -> bool {
        false
    }

    /// Configura timer em modo one-shot.
    // TODO: Revisar no futuro
    #[allow(unused)]
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

impl SerialPrintTo for InterruptControllerType {
    fn serial_print_to(&self, s: &mut SerialPort) {
        match self {
            Self::Pic => s.write_str("PIC"),
            Self::LocalApic => s.write_str("Local APIC"),
            Self::IoApic => s.write_str("I/O APIC"),
            Self::Msi => s.write_str("MSI"),
            Self::MsiX => s.write_str("MSI-X"),
        }
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

impl SerialPrintTo for PowerState {
    fn serial_print_to(&self, s: &mut SerialPort) {
        match self {
            Self::Running => s.write_str("Running"),
            Self::Standby => s.write_str("Standby"),
            Self::SuspendToRam => s.write_str("Suspend to RAM"),
            Self::Hibernate => s.write_str("Hibernate"),
            Self::Off => s.write_str("Off"),
        }
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

impl SerialPrintTo for ResetMethod {
    fn serial_print_to(&self, s: &mut SerialPort) {
        match self {
            Self::Acpi => s.write_str("ACPI"),
            Self::Keyboard => s.write_str("Keyboard"),
            Self::TripleFault => s.write_str("Triple Fault"),
            Self::Pci => s.write_str("PCI"),
        }
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

impl SerialPrintTo for DmaChannel {
    fn serial_print_to(&self, s: &mut SerialPort) {
        s.write_str("Channel ");
        s.write_hex(*self as u64);
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

impl SerialPrintTo for DmaMode {
    fn serial_print_to(&self, s: &mut SerialPort) {
        match self {
            Self::Read => s.write_str("Read"),
            Self::Write => s.write_str("Write"),
            Self::Verify => s.write_str("Verify"),
        }
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

impl SerialPrintTo for DmaError {
    fn serial_print_to(&self, s: &mut SerialPort) {
        match self {
            Self::InvalidChannel => s.write_str("Invalid Channel"),
            Self::BoundaryCross => s.write_str("Boundary Cross"),
            Self::AddressTooHigh => s.write_str("Address Too High"),
            Self::ChannelBusy => s.write_str("Channel Busy"),
        }
    }
}
