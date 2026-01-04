//! # HID Report Parser
//!
//! Parser de Report Descriptors e Reports HID.
//!
//! ## STUB:
//! Parsing básico implementado. Suporte completo pendente.

use super::types::*;
// TODO: Revisar no futuro
#[allow(unused_imports)]
use super::usage::*;
use alloc::vec::Vec;

// =============================================================================
// PARSING DE REPORT DESCRIPTOR
// =============================================================================

/// Parseia um Report Descriptor.
///
/// ## STUB:
/// Parsing básico, não suporta todos os itens.
pub fn parse_descriptor(data: &[u8]) -> Option<HidReportDescriptor> {
    crate::kwarn!("(HID) parse_descriptor() parcialmente implementado");

    let mut descriptor = HidReportDescriptor::default();
    let mut state = HidReportState::default();
    let mut usages: Vec<u16> = Vec::new();
    let mut bit_offset: u32 = 0;

    let mut i = 0;
    while i < data.len() {
        let prefix = data[i];
        let size = (prefix & 0x03) as usize;
        let item_type = HidItemType::from_byte(prefix);
        let tag = prefix & 0xFC;

        // Lê dados do item
        let item_data = if size > 0 && i + size < data.len() {
            &data[i + 1..i + 1 + size]
        } else {
            &[]
        };

        let value = match size {
            1 => item_data.get(0).copied().unwrap_or(0) as u32,
            2 => {
                let lo = item_data.get(0).copied().unwrap_or(0) as u32;
                let hi = item_data.get(1).copied().unwrap_or(0) as u32;
                lo | (hi << 8)
            }
            4 => {
                let mut v = 0u32;
                for (j, &b) in item_data.iter().enumerate() {
                    v |= (b as u32) << (j * 8);
                }
                v
            }
            _ => 0,
        };

        match item_type {
            HidItemType::Main => {
                match tag {
                    0x80 => {
                        // Input
                        for usage in usages.drain(..) {
                            descriptor.input_fields.push(HidReportField {
                                usage_page: state.usage_page,
                                usage,
                                bit_offset,
                                bit_size: state.report_size,
                                logical_min: state.logical_min,
                                logical_max: state.logical_max,
                                flags: value as u8,
                            });
                            bit_offset += state.report_size;
                        }
                    }
                    0x90 => {
                        // Output
                        for usage in usages.drain(..) {
                            descriptor.output_fields.push(HidReportField {
                                usage_page: state.usage_page,
                                usage,
                                bit_offset,
                                bit_size: state.report_size,
                                logical_min: state.logical_min,
                                logical_max: state.logical_max,
                                flags: value as u8,
                            });
                            bit_offset += state.report_size;
                        }
                    }
                    0xB0 => {
                        // Feature
                        for usage in usages.drain(..) {
                            descriptor.feature_fields.push(HidReportField {
                                usage_page: state.usage_page,
                                usage,
                                bit_offset,
                                bit_size: state.report_size,
                                logical_min: state.logical_min,
                                logical_max: state.logical_max,
                                flags: value as u8,
                            });
                            bit_offset += state.report_size;
                        }
                    }
                    0xA0 => {
                        // Collection
                        usages.clear();
                    }
                    0xC0 => {
                        // End Collection
                    }
                    _ => {}
                }
            }
            HidItemType::Global => match tag {
                0x04 => state.usage_page = value as u16,
                0x14 => state.logical_min = value as i32,
                0x24 => state.logical_max = value as i32,
                0x34 => state.physical_min = value as i32,
                0x44 => state.physical_max = value as i32,
                0x74 => state.report_size = value,
                0x84 => {
                    state.report_id = value as u8;
                    if !descriptor.report_ids.contains(&(value as u8)) {
                        descriptor.report_ids.push(value as u8);
                    }
                }
                0x94 => state.report_count = value,
                _ => {}
            },
            HidItemType::Local => {
                match tag {
                    0x08 => usages.push(value as u16),
                    0x18 => {
                        // Usage Minimum
                        // TODO: Revisar no futuro
                        #[allow(unused)]
                        let min = value as u16;
                        // Assume Usage Maximum vem logo depois
                    }
                    0x28 => {
                        // Usage Maximum - adiciona range
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        i += 1 + size;
    }

    Some(descriptor)
}

// =============================================================================
// PARSING DE REPORTS
// =============================================================================

/// Parseia um Input Report.
///
/// ## STUB:
/// Parsing básico.
pub fn parse_report(descriptor: &HidReportDescriptor, data: &[u8]) -> Vec<(u16, u16, i32)> {
    let mut results = Vec::new();

    for field in &descriptor.input_fields {
        let value = extract_bits(data, field.bit_offset, field.bit_size);

        // Aplica extensão de sinal se necessário
        let signed_value = if field.logical_min < 0 {
            let max_val = 1i32 << (field.bit_size - 1);
            if value as i32 >= max_val {
                value as i32 - (1i32 << field.bit_size)
            } else {
                value as i32
            }
        } else {
            value as i32
        };

        results.push((field.usage_page, field.usage, signed_value));
    }

    results
}

/// Extrai bits de um buffer.
fn extract_bits(data: &[u8], bit_offset: u32, bit_size: u32) -> u32 {
    let mut value = 0u32;

    for i in 0..bit_size {
        let bit_pos = bit_offset + i;
        let byte_idx = (bit_pos / 8) as usize;
        let bit_idx = (bit_pos % 8) as u8;

        if byte_idx < data.len() {
            if (data[byte_idx] & (1 << bit_idx)) != 0 {
                value |= 1 << i;
            }
        }
    }

    value
}
