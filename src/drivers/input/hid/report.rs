//! # HID Report Parser
//!
//! Responsável por interpretar os Report Descriptors do HID.
//! Transforma bytes brutos em eventos de input estruturados.

use super::usage::UsagePage;

#[derive(Debug, Clone, Copy)]
pub enum HidItemType {
    Main = 0,
    Global = 1,
    Local = 2,
    Reserved = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct HidItem {
    pub size: u8,
    pub type_: HidItemType,
    pub tag: u8,
    pub value: u32,
}

pub struct HidReportParser {
    // Estado do parser durante a interpretação do descritor
    global_usage_page: UsagePage,
    report_size: u32,
    report_count: u32,
    // ... outros campos de estado (Logical Min/Max, etc)
}

impl HidReportParser {
    pub fn new() -> Self {
        Self {
            global_usage_page: UsagePage::Undefined,
            report_size: 0,
            report_count: 0,
        }
    }

    /// Faz o paring de um item do descritor HID
    pub fn parse_item(&mut self, data: &[u8]) -> Option<(HidItem, usize)> {
        if data.is_empty() {
            return None;
        }

        let b0 = data[0];
        let size = match b0 & 0x03 {
            3 => 4,
            s => s,
        } as usize;

        if data.len() < 1 + size {
            return None;
        }

        let type_ = match (b0 >> 2) & 0x03 {
            0 => HidItemType::Main,
            1 => HidItemType::Global,
            2 => HidItemType::Local,
            _ => HidItemType::Reserved,
        };

        let tag = b0 >> 4;
        let mut value = 0u32;
        for i in 0..size {
            value |= (data[1 + i] as u32) << (i * 8);
        }

        Some((
            HidItem {
                size: size as u8,
                type_,
                tag,
                value,
            },
            1 + size,
        ))
    }

    /// Processa um relatório de entrada (Input Report) recebido do hardware
    pub fn process_input_report(&self, report_data: &[u8]) {
        // TODO: Mapear os bits do report_data para os usages definidos no descritor
        crate::ktrace!(
            "(HID) Processando Input Report de {} bytes",
            report_data.len() as u64
        );
    }
}
