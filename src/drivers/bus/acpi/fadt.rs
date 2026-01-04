//! # FADT Parser (Fixed ACPI Description Table)
//!
//! Este arquivo parseia a tabela **FADT** (também chamada FACP)
//! que contém informações fixas sobre o sistema e gerenciamento de energia.
//!
//! ## Informações Contidas:
//! - Endereços dos registradores de PM (Power Management)
//! - Endereço da DSDT
//! - Flags de capacidades do sistema
//! - PM Timer info
//!
//! ## STUB:
//! Estruturas definidas, parsing não implementado.

use super::tables::GenericAddress;
use super::AcpiTableHeader;

// =============================================================================
// ESTRUTURA DA FADT
// =============================================================================

/// Fixed ACPI Description Table
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Fadt {
    pub header: AcpiTableHeader,

    /// Endereço físico da FACS
    pub firmware_ctrl: u32,
    /// Endereço físico da DSDT
    pub dsdt: u32,

    /// Reservado (era int_model em ACPI 1.0)
    pub reserved: u8,
    /// Perfil de gerenciamento de energia preferido
    pub preferred_pm_profile: u8,
    /// System Control Interrupt
    pub sci_interrupt: u16,
    /// Porta SMI Command
    pub smi_command: u32,
    /// Valor para habilitar ACPI
    pub acpi_enable: u8,
    /// Valor para desabilitar ACPI
    pub acpi_disable: u8,
    /// Valor para entrar em S4BIOS
    pub s4_bios_request: u8,
    /// Valor para ativar P-states
    pub pstate_control: u8,

    /// Porta de evento PM1a
    pub pm1a_event_block: u32,
    /// Porta de evento PM1b
    pub pm1b_event_block: u32,
    /// Porta de controle PM1a
    pub pm1a_control_block: u32,
    /// Porta de controle PM1b
    pub pm1b_control_block: u32,
    /// Porta de controle PM2
    pub pm2_control_block: u32,
    /// Porta do PM Timer
    pub pm_timer_block: u32,
    /// Porta GPE0
    pub gpe0_block: u32,
    /// Porta GPE1
    pub gpe1_block: u32,

    /// Tamanho do evento PM1
    pub pm1_event_length: u8,
    /// Tamanho do controle PM1
    pub pm1_control_length: u8,
    /// Tamanho do controle PM2
    pub pm2_control_length: u8,
    /// Tamanho do PM Timer
    pub pm_timer_length: u8,
    /// Tamanho do GPE0
    pub gpe0_length: u8,
    /// Tamanho do GPE1
    pub gpe1_length: u8,
    /// Base do GPE1
    pub gpe1_base: u8,
    /// Comando para C-state
    pub cstate_control: u8,
    /// Latência máxima para C2
    pub worst_c2_latency: u16,
    /// Latência máxima para C3
    pub worst_c3_latency: u16,
    /// Tamanho do flush para C3
    pub flush_size: u16,
    /// Stride do flush para C3
    pub flush_stride: u16,
    /// Offset do duty cycle
    pub duty_offset: u8,
    /// Largura do duty cycle
    pub duty_width: u8,
    /// Dia do alarme RTC
    pub day_alarm: u8,
    /// Mês do alarme RTC
    pub month_alarm: u8,
    /// Século RTC
    pub century: u8,

    /// Flags de boot da arquitetura IA-PC
    pub iapc_boot_arch: u16,
    /// Reservado
    pub reserved2: u8,
    /// Flags
    pub flags: u32,

    /// Registrador de reset
    pub reset_reg: GenericAddress,
    /// Valor de reset
    pub reset_value: u8,
    /// Controle ARM Boot Arch
    pub arm_boot_arch: u16,
    /// Versão FADT Minor
    pub fadt_minor_version: u8,

    // Extensões ACPI 2.0+
    /// FACS 64-bit
    pub x_firmware_ctrl: u64,
    /// DSDT 64-bit
    pub x_dsdt: u64,

    /// PM1a Event Block 64-bit
    pub x_pm1a_event_block: GenericAddress,
    /// PM1b Event Block 64-bit
    pub x_pm1b_event_block: GenericAddress,
    /// PM1a Control Block 64-bit
    pub x_pm1a_control_block: GenericAddress,
    /// PM1b Control Block 64-bit
    pub x_pm1b_control_block: GenericAddress,
    /// PM2 Control Block 64-bit
    pub x_pm2_control_block: GenericAddress,
    /// PM Timer Block 64-bit
    pub x_pm_timer_block: GenericAddress,
    /// GPE0 Block 64-bit
    pub x_gpe0_block: GenericAddress,
    /// GPE1 Block 64-bit
    pub x_gpe1_block: GenericAddress,
}

// =============================================================================
// FLAGS DA FADT
// =============================================================================

/// WBINVD instruction works correctly
pub const FADT_FLAG_WBINVD: u32 = 1 << 0;
/// WBINVD flushes all caches
pub const FADT_FLAG_WBINVD_FLUSH: u32 = 1 << 1;
/// Processor C1 state supported
pub const FADT_FLAG_PROC_C1: u32 = 1 << 2;
/// C2 works on MP systems
pub const FADT_FLAG_P_LVL2_UP: u32 = 1 << 3;
/// Power button is control method
pub const FADT_FLAG_PWR_BUTTON: u32 = 1 << 4;
/// Sleep button is control method
pub const FADT_FLAG_SLP_BUTTON: u32 = 1 << 5;
/// RTC wake status not in fixed register space
pub const FADT_FLAG_FIX_RTC: u32 = 1 << 6;
/// RTC can wake from S4
pub const FADT_FLAG_RTC_S4: u32 = 1 << 7;
/// TMR_VAL is 32 bits
pub const FADT_FLAG_TMR_VAL_EXT: u32 = 1 << 8;
/// DCK_EN is supported
pub const FADT_FLAG_DCK_CAP: u32 = 1 << 9;
/// System reset via FADT supported
pub const FADT_FLAG_RESET_REG_SUP: u32 = 1 << 10;
/// Sealed case
pub const FADT_FLAG_SEALED_CASE: u32 = 1 << 11;
/// Headless system
pub const FADT_FLAG_HEADLESS: u32 = 1 << 12;
/// Execute native instr after SLP_TYPx
pub const FADT_FLAG_CPU_SW_SLP: u32 = 1 << 13;
/// Uses platform RTC clock
pub const FADT_FLAG_PCI_EXP_WAK: u32 = 1 << 14;
/// Platform supports S1
pub const FADT_FLAG_USE_PLATFORM_CLOCK: u32 = 1 << 15;
/// RTC_STS valid on S4 wake
pub const FADT_FLAG_S4_RTC_STS_VALID: u32 = 1 << 16;
/// Remote power on capable
pub const FADT_FLAG_REMOTE_POWER_ON_CAP: u32 = 1 << 17;
/// Force APIC cluster model
pub const FADT_FLAG_FORCE_APIC_CLUSTER: u32 = 1 << 18;
/// Force physical APIC mode
pub const FADT_FLAG_FORCE_APIC_PHYS: u32 = 1 << 19;
/// Hardware-reduced ACPI
pub const FADT_FLAG_HW_REDUCED_ACPI: u32 = 1 << 20;
/// Low power S0 idle
pub const FADT_FLAG_LOW_POWER_S0: u32 = 1 << 21;

// =============================================================================
// RESULTADO DO PARSING
// =============================================================================

/// Informações extraídas da FADT.
#[derive(Debug, Clone, Default)]
pub struct FadtInfo {
    /// Endereço da DSDT
    pub dsdt_address: u64,

    /// Endereço do PM Timer
    pub pm_timer_address: u64,
    /// PM Timer é 32 bits?
    pub pm_timer_32bit: bool,

    /// Interrupção SCI
    pub sci_interrupt: u16,

    /// Suporta reset via ACPI?
    pub reset_supported: bool,
    /// Endereço do reset register
    pub reset_address: u64,
    /// Valor para reset
    pub reset_value: u8,

    /// Hardware-reduced ACPI?
    pub hw_reduced: bool,

    /// Flags
    pub flags: u32,
}

// =============================================================================
// FUNÇÕES DE PARSING
// =============================================================================

/// Parseia a tabela FADT.
///
/// ## STUB:
/// Não implementado.
pub fn parse(_fadt_address: u64) -> FadtInfo {
    crate::kwarn!("(FADT) parse() não implementado");

    // TODO: Implementar

    FadtInfo::default()
}

/// Realiza reset do sistema via ACPI (se suportado).
///
/// ## STUB:
/// Não implementado.
pub fn acpi_reset() -> bool {
    crate::kwarn!("(FADT) acpi_reset() não implementado");
    false
}
