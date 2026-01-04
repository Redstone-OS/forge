//! # Touchpad Utility Functions
//!
//! Funções de baixo nível para comunicação I2C-HID e detecção de hardware.

use super::hid::{self, commands as hid_cmd, regs as hid_regs, HidDescriptor, TouchpadType};
use super::i2c::{I2cController, I2cError};

/// Tenta detectar um touchpad nos endereços conhecidos
pub fn detect_touchpad(i2c: &I2cController) -> (u8, TouchpadType) {
    // Tentar Synaptics primeiro
    i2c.set_target(hid::SYNAPTICS_ADDR);
    if probe_device(i2c) {
        return (hid::SYNAPTICS_ADDR, TouchpadType::Synaptics);
    }

    // Tentar ELAN
    i2c.set_target(hid::ELAN_ADDR);
    if probe_device(i2c) {
        return (hid::ELAN_ADDR, TouchpadType::Elan);
    }

    (0, TouchpadType::Unknown)
}

/// Tenta ler algo do dispositivo para verificar se existe
pub fn probe_device(i2c: &I2cController) -> bool {
    let reg_bytes = (hid_regs::HID_DESC_REGISTER as u16).to_le_bytes();
    let mut buf = [0u8; 2];
    i2c.write_read(&reg_bytes, &mut buf).is_ok()
}

/// Lê o HID Descriptor do dispositivo
pub fn read_hid_descriptor(i2c: &I2cController) -> Option<HidDescriptor> {
    let reg_bytes = (hid_regs::HID_DESC_REGISTER as u16).to_le_bytes();
    let mut buf = [0u8; 30];

    if i2c.write_read(&reg_bytes, &mut buf).is_err() {
        return None;
    }

    let desc = HidDescriptor {
        desc_length: u16::from_le_bytes([buf[0], buf[1]]),
        bcd_version: u16::from_le_bytes([buf[2], buf[3]]),
        report_desc_length: u16::from_le_bytes([buf[4], buf[5]]),
        report_desc_register: u16::from_le_bytes([buf[6], buf[7]]),
        input_register: u16::from_le_bytes([buf[8], buf[9]]),
        max_input_length: u16::from_le_bytes([buf[10], buf[11]]),
        output_register: u16::from_le_bytes([buf[12], buf[13]]),
        max_output_length: u16::from_le_bytes([buf[14], buf[15]]),
        command_register: u16::from_le_bytes([buf[16], buf[17]]),
        data_register: u16::from_le_bytes([buf[18], buf[19]]),
        vendor_id: u16::from_le_bytes([buf[20], buf[21]]),
        product_id: u16::from_le_bytes([buf[22], buf[23]]),
        version_id: u16::from_le_bytes([buf[24], buf[25]]),
        reserved: u32::from_le_bytes([buf[26], buf[27], buf[28], buf[29]]),
    };

    if desc.is_valid() {
        Some(desc)
    } else {
        None
    }
}

/// Envia um comando HID
pub fn send_command(
    i2c: &I2cController,
    desc: &HidDescriptor,
    opcode: u8,
    param: u8,
) -> Result<(), I2cError> {
    let cmd_reg = desc.command_register.to_le_bytes();
    let cmd_data = [cmd_reg[0], cmd_reg[1], opcode | (param << 4), 0];
    i2c.write(&cmd_data)
}
