/// Arquivo: x86_64/acpi/fadt.rs
///
/// Propósito: Parsing da Fixed ACPI Description Table (FADT).
/// Esta tabela contém informações estáticas sobre o gerenciamento de energia (Power Management),
/// ponteiros para outras tabelas importantes (DSDT, FACS) e configurações de Boot (Reset Register).
///
/// Detalhes de Implementação:
/// - Define a estrutura `Fadt` compatível com ACPI 2.0+.
/// - Define `GenericAddressStructure` (GAS) usada para descrever registradores em I/O ou MMIO.
/// - Essencial para: Shutdown, Reboot, Habilitar ACPI Mode.

// ACPI FADT (Fixed ACPI Description Table)

/// Generic Address Structure (GAS)
/// Usado pelo ACPI para descrever a localização de registradores (IO, Memory, PCI Config, etc.)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GenericAddressStructure {
    pub address_space_id: u8, // 0=System Mem, 1=System IO, 2=PCI Config
    pub register_bit_width: u8,
    pub register_bit_offset: u8,
    pub access_size: u8,
    pub address: u64,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Fadt {
    pub signature: [u8; 4], // "FACP"
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,

    // ACPI 1.0 Fields
    pub firmware_ctrl: u32, // Ponteiro físico para FACS
    pub dsdt: u32,          // Ponteiro físico para DSDT

    pub reserved: u8,
    pub preferred_pm_profile: u8, // Desktop, Mobile, Server, etc.
    pub sci_int: u16,             // Interrupção System Control Interrupt (GSI)
    pub smi_cmd: u32,             // Porta IO para comandos SMI (System Management Interrupt)
    pub acpi_enable: u8,          // Valor para escrever em smi_cmd para habilitar ACPI
    pub acpi_disable: u8,         // Valor para desabilitar
    pub s4bios_req: u8,
    pub pstate_cnt: u8,

    pub pm1a_evt_blk: u32, // Registradores de Eventos PM1a
    pub pm1b_evt_blk: u32,
    pub pm1a_cnt_blk: u32, // Registradores de Controle PM1a (Shutdown/Sleep)
    pub pm1b_cnt_blk: u32,
    pub pm2_cnt_blk: u32,
    pub pm_tmr_blk: u32, // Power Management Timer

    pub gpe0_blk: u32,
    pub gpe1_blk: u32,
    pub pm1_evt_len: u8,
    pub pm1_cnt_len: u8,
    pub pm2_cnt_len: u8,
    pub pm_tmr_len: u8,
    pub gpe0_blk_len: u8,
    pub gpe1_blk_len: u8,
    pub gpe1_base: u8,
    pub cst_cnt: u8,
    pub p_lvl2_lat: u16,
    pub p_lvl3_lat: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alrm: u8,
    pub mon_alrm: u8,
    pub century: u8, // Offset no CMOS para o século (RTC fix)

    // ACPI 2.0+ Fields
    pub iapc_boot_arch: u16, // Flags de arquitetura de boot (Legacy Devices, 8042, etc.)
    pub reserved2: u8,
    pub flags: u32,

    pub reset_reg: GenericAddressStructure, // Registrador para Reset via ACPI
    pub reset_value: u8,                    // Valor para escrever no reset_reg

    pub reserved3: [u8; 3],

    // Endereços 64-bit (X Fields) - Usados se os de 32-bit forem 0 ou insuficientes
    pub x_firmware_ctrl: u64,
    pub x_dsdt: u64,

    pub x_pm1a_evt_blk: GenericAddressStructure,
    pub x_pm1b_evt_blk: GenericAddressStructure,
    pub x_pm1a_cnt_blk: GenericAddressStructure,
    pub x_pm1b_cnt_blk: GenericAddressStructure,
    pub x_pm2_cnt_blk: GenericAddressStructure,
    pub x_pm_tmr_blk: GenericAddressStructure,
    pub x_gpe0_blk: GenericAddressStructure,
    pub x_gpe1_blk: GenericAddressStructure,
}
