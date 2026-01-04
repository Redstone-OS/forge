//! # FADT (Fixed ACPI Description Table)
//!
//! Contém informações sobre temporizadores, gerenciamento de energia e reset
//! de hardware.

use super::tables::SdtHeader;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct FadtHeader {
    pub header: SdtHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,
    pub reserved: u8,
    pub preferred_pm_profile: u8,
    pub sci_interrupt: u16,
    pub smi_command_port: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_control: u8,
    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
    pub gpe0_block: u32,
    pub gpe1_block: u32,
    pub pm1_event_length: u8,
    pub pm1_control_length: u8,
    pub pm2_control_length: u8,
    pub pm_timer_length: u8,
    pub gpe0_length: u8,
    pub gpe1_length: u8,
    pub gpe1_base: u8,
    pub cstate_control: u8,
    pub worst_c2_latency: u16,
    pub worst_c3_latency: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alarm: u8,
    pub month_alarm: u8,
    pub century: u8,
    pub iapc_boot_arch: u16, // Flags de arquitetura de boot (Ex: tem 8042?)
    pub reserved2: u8,
    pub flags: u32,
    pub reset_reg: [u8; 12], // Registro de Reset (Generic Address Structure)
    pub reset_value: u8,
}

impl FadtHeader {
    /// Verifica se o sistema possui um controlador de teclado PS/2 (8042)
    pub fn has_8042(&self) -> bool {
        (self.iapc_boot_arch & (1 << 1)) != 0
    }

    /// Tenta realizar o reset do sistema via ACPI
    pub fn reset_system(&self) {
        if (self.flags & (1 << 10)) != 0 {
            // O bit 10 das flags indica suporte a reset via registro
            // TODO: Implementar escrita no registro genérico definido em reset_reg
            crate::kwarn!("(ACPI) Reset via FADT solicitado, mas não implementado.");
        }
    }
}
