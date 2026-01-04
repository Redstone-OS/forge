//! # HID Types and Constants
//!
//! Tipos e constantes do protocolo HID.

// =============================================================================
// ITEM TYPES
// =============================================================================

/// Tipo de item no Report Descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidItemType {
    /// Main item (Input, Output, Feature, Collection, End Collection).
    Main,
    /// Global item (Usage Page, Logical Min/Max, etc).
    Global,
    /// Local item (Usage, Designator, String, etc).
    Local,
    /// Reservado.
    Reserved,
}

impl HidItemType {
    pub fn from_byte(byte: u8) -> Self {
        match (byte >> 2) & 0x03 {
            0 => Self::Main,
            1 => Self::Global,
            2 => Self::Local,
            _ => Self::Reserved,
        }
    }
}

// =============================================================================
// MAIN ITEMS
// =============================================================================

/// Tags de Main items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidMainTag {
    Input = 0x80,
    Output = 0x90,
    Feature = 0xB0,
    Collection = 0xA0,
    EndCollection = 0xC0,
}

/// Tipo de Collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidCollectionType {
    Physical = 0x00,
    Application = 0x01,
    Logical = 0x02,
    Report = 0x03,
    NamedArray = 0x04,
    UsageSwitch = 0x05,
    UsageModifier = 0x06,
}

// =============================================================================
// GLOBAL ITEMS
// =============================================================================

/// Tags de Global items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidGlobalTag {
    UsagePage = 0x04,
    LogicalMinimum = 0x14,
    LogicalMaximum = 0x24,
    PhysicalMinimum = 0x34,
    PhysicalMaximum = 0x44,
    UnitExponent = 0x54,
    Unit = 0x64,
    ReportSize = 0x74,
    ReportId = 0x84,
    ReportCount = 0x94,
    Push = 0xA4,
    Pop = 0xB4,
}

// =============================================================================
// LOCAL ITEMS
// =============================================================================

/// Tags de Local items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidLocalTag {
    Usage = 0x08,
    UsageMinimum = 0x18,
    UsageMaximum = 0x28,
    DesignatorIndex = 0x38,
    DesignatorMinimum = 0x48,
    DesignatorMaximum = 0x58,
    StringIndex = 0x78,
    StringMinimum = 0x88,
    StringMaximum = 0x98,
    Delimiter = 0xA8,
}

// =============================================================================
// REPORT DESCRIPTOR STATE
// =============================================================================

/// Estado durante parsing de Report Descriptor.
#[derive(Debug, Clone, Default)]
pub struct HidReportState {
    /// Usage Page atual.
    pub usage_page: u16,
    /// Logical Minimum.
    pub logical_min: i32,
    /// Logical Maximum.
    pub logical_max: i32,
    /// Physical Minimum.
    pub physical_min: i32,
    /// Physical Maximum.
    pub physical_max: i32,
    /// Report Size (bits).
    pub report_size: u32,
    /// Report Count.
    pub report_count: u32,
    /// Report ID atual.
    pub report_id: u8,
}

// =============================================================================
// REPORT FIELD
// =============================================================================

/// Campo em um Report.
#[derive(Debug, Clone)]
pub struct HidReportField {
    /// Usage Page.
    pub usage_page: u16,
    /// Usage.
    pub usage: u16,
    /// Offset em bits no report.
    pub bit_offset: u32,
    /// Tamanho em bits.
    pub bit_size: u32,
    /// Logical Minimum.
    pub logical_min: i32,
    /// Logical Maximum.
    pub logical_max: i32,
    /// Flags (Constant, Variable, Relative, etc).
    pub flags: u8,
}

// =============================================================================
// PARSED REPORT DESCRIPTOR
// =============================================================================

/// Report Descriptor parseado.
#[derive(Debug, Clone, Default)]
pub struct HidReportDescriptor {
    /// Campos de Input.
    pub input_fields: alloc::vec::Vec<HidReportField>,
    /// Campos de Output.
    pub output_fields: alloc::vec::Vec<HidReportField>,
    /// Campos de Feature.
    pub feature_fields: alloc::vec::Vec<HidReportField>,
    /// Report IDs usados.
    pub report_ids: alloc::vec::Vec<u8>,
}

// =============================================================================
// FLAGS DE INPUT/OUTPUT/FEATURE
// =============================================================================

/// Data (vs Constant).
pub const HID_DATA: u8 = 0x00;
/// Constant (vs Data).
pub const HID_CONSTANT: u8 = 0x01;
/// Array (vs Variable).
pub const HID_ARRAY: u8 = 0x00;
/// Variable (vs Array).
pub const HID_VARIABLE: u8 = 0x02;
/// Absolute (vs Relative).
pub const HID_ABSOLUTE: u8 = 0x00;
/// Relative (vs Absolute).
pub const HID_RELATIVE: u8 = 0x04;
/// No Wrap.
pub const HID_NO_WRAP: u8 = 0x00;
/// Wrap.
pub const HID_WRAP: u8 = 0x08;
/// Linear.
pub const HID_LINEAR: u8 = 0x00;
/// Non-Linear.
pub const HID_NON_LINEAR: u8 = 0x10;
/// Preferred State.
pub const HID_PREFERRED_STATE: u8 = 0x00;
/// No Preferred.
pub const HID_NO_PREFERRED: u8 = 0x20;
/// No Null Position.
pub const HID_NO_NULL: u8 = 0x00;
/// Null State.
pub const HID_NULL_STATE: u8 = 0x40;
/// Non Volatile.
pub const HID_NON_VOLATILE: u8 = 0x00;
/// Volatile.
pub const HID_VOLATILE: u8 = 0x80;
