//! # HID Internal Types
//!
//! Tipos internos para o gerenciamento de coleções e campos HID.

use super::usage::HidUsage;

pub enum HidCollectionType {
    Physical = 0x00,
    Application = 0x01,
    Logical = 0x02,
    Report = 0x03,
    NamedArray = 0x04,
    UsageSwitch = 0x05,
    UsageModifier = 0x06,
}

pub struct HidField {
    pub usage: HidUsage,
    pub logical_min: i32,
    pub logical_max: i32,
    pub report_size: u32,
    pub report_count: u32,
    pub offset: u32, // Offset em bits no report
}

pub struct HidCollection {
    pub type_: HidCollectionType,
    pub usage: HidUsage,
    pub fields: alloc::vec::Vec<HidField>,
}
