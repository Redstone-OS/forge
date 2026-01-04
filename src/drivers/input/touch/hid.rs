//! # Constantes e Registradores I2C-HID
//!
//! Definições do protocolo HID over I2C conforme especificação Microsoft.
//!
//! ## Referência
//! - [HID over I2C Protocol Specification](https://docs.microsoft.com/en-us/windows-hardware/design/component-guidelines/hid-over-i2c-protocol-specification)

/// Endereço I2C padrão do touchpad Synaptics
pub const SYNAPTICS_ADDR: u8 = 0x2C;

/// Endereço I2C padrão do touchpad ELAN
pub const ELAN_ADDR: u8 = 0x15;

/// Registradores HID over I2C
pub mod regs {
    /// HID Descriptor Register (offset onde buscar o HID descriptor)
    pub const HID_DESC_REGISTER: u16 = 0x0001;
    /// Report Descriptor Register (offset do report descriptor)
    pub const REPORT_DESC_REGISTER: u16 = 0x0020;
    /// Input Register (entrada de dados)
    pub const INPUT_REGISTER: u16 = 0x0021;
    /// Output Register (saída de dados)
    pub const OUTPUT_REGISTER: u16 = 0x0022;
    /// Command Register (comandos)
    pub const COMMAND_REGISTER: u16 = 0x0023;
    /// Data Register (dados de comando)
    pub const DATA_REGISTER: u16 = 0x0024;
}

/// Opcodes de comandos HID over I2C
pub mod commands {
    /// Reset do dispositivo
    pub const RESET: u8 = 0x01;
    /// Get Report
    pub const GET_REPORT: u8 = 0x02;
    /// Set Report
    pub const SET_REPORT: u8 = 0x03;
    /// Get Idle
    pub const GET_IDLE: u8 = 0x04;
    /// Set Idle
    pub const SET_IDLE: u8 = 0x05;
    /// Get Protocol
    pub const GET_PROTOCOL: u8 = 0x06;
    /// Set Protocol
    pub const SET_PROTOCOL: u8 = 0x07;
    /// Set Power (Low/On)
    pub const SET_POWER: u8 = 0x08;
}

/// Estados de power
pub mod power {
    /// Power On (full power)
    pub const ON: u8 = 0x00;
    /// Sleep (low power)
    pub const SLEEP: u8 = 0x01;
}

/// Tipos de Report
pub mod report_type {
    /// Input Report
    pub const INPUT: u8 = 0x01;
    /// Output Report
    pub const OUTPUT: u8 = 0x02;
    /// Feature Report
    pub const FEATURE: u8 = 0x03;
}

/// Fabricantes suportados
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchpadType {
    Unknown,
    Synaptics,
    Elan,
}

/// HID Descriptor (30 bytes conforme especificação)
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct HidDescriptor {
    /// Tamanho deste descriptor (deve ser 30)
    pub desc_length: u16,
    /// Versão do BCD do protocolo (0x0100 = 1.0)
    pub bcd_version: u16,
    /// Tamanho do Report Descriptor
    pub report_desc_length: u16,
    /// Endereço do Report Descriptor Register
    pub report_desc_register: u16,
    /// Endereço do Input Register
    pub input_register: u16,
    /// Tamanho máximo do Input Report
    pub max_input_length: u16,
    /// Endereço do Output Register
    pub output_register: u16,
    /// Tamanho máximo do Output Report
    pub max_output_length: u16,
    /// Endereço do Command Register
    pub command_register: u16,
    /// Endereço do Data Register
    pub data_register: u16,
    /// Vendor ID
    pub vendor_id: u16,
    /// Product ID
    pub product_id: u16,
    /// Versão do dispositivo
    pub version_id: u16,
    /// Reservado (deve ser 0)
    pub reserved: u32,
}

impl HidDescriptor {
    /// Verifica se o descriptor é válido
    pub fn is_valid(&self) -> bool {
        self.desc_length == 30 && self.bcd_version >= 0x0100
    }
}

/// Finger data para touchpad (posição de um dedo)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FingerData {
    /// ID do contato (0-9 tipicamente)
    pub contact_id: u8,
    /// Tip switch (dedo tocando)
    pub tip: bool,
    /// In range (dedo próximo)
    pub in_range: bool,
    /// Posição X
    pub x: u16,
    /// Posição Y
    pub y: u16,
    /// Pressão (se suportado)
    pub pressure: u8,
}

/// State do touchpad
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct TouchpadState {
    /// Número de dedos ativos
    pub finger_count: u8,
    /// Dados de cada dedo (até 5)
    pub fingers: [FingerData; 5],
    /// Botão esquerdo
    pub button_left: bool,
    /// Botão direito
    pub button_right: bool,
    /// Posição X do cursor (calculada)
    pub cursor_x: i32,
    /// Posição Y do cursor (calculada)
    pub cursor_y: i32,
    /// Delta X
    pub delta_x: i32,
    /// Delta Y
    pub delta_y: i32,
    /// Largura da tela
    pub screen_width: i32,
    /// Altura da tela
    pub screen_height: i32,
}
