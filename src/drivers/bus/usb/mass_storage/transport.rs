//! # Transporte Bulk-Only (BOT) para USB Mass Storage

use crate::drivers::usb::xhci;
use core::sync::atomic::{AtomicU32, Ordering};

static GLOBAL_TAG: AtomicU32 = AtomicU32::new(0x1000);

#[repr(C, packed)]
pub struct CommandBlockWrapper {
    pub signature: u32,
    pub tag: u32,
    pub data_transfer_len: u32,
    pub flags: u8,
    pub lun: u8,
    pub cb_len: u8,
    pub cb: [u8; 16],
}

#[repr(C, packed)]
pub struct CommandStatusWrapper {
    pub signature: u32,
    pub tag: u32,
    pub data_residue: u32,
    pub status: u8,
}

pub struct BulkOnlyTransport {
    pub slot_id: u8,
    pub ep_in: u8,
    pub ep_out: u8,
    tag_counter: AtomicU32,
}

impl BulkOnlyTransport {
    pub fn new(slot_id: u8, ep_in: u8, ep_out: u8) -> Self {
        Self {
            slot_id,
            ep_in,
            ep_out,
            tag_counter: AtomicU32::new(GLOBAL_TAG.fetch_add(100, Ordering::SeqCst)),
        }
    }

    pub fn send_command(&self, scsi_cmd: &[u8], data: Option<&mut [u8]>, is_in: bool) -> bool {
        let current_tag = self.tag_counter.fetch_add(1, Ordering::SeqCst);

        let mut cbw = CommandBlockWrapper {
            signature: 0x43425355,
            tag: current_tag,
            data_transfer_len: data.as_ref().map(|d| d.len() as u32).unwrap_or(0),
            flags: if is_in { 0x80 } else { 0x00 },
            lun: 0,
            cb_len: scsi_cmd.len() as u8,
            cb: [0; 16],
        };
        cbw.cb[..scsi_cmd.len()].copy_from_slice(scsi_cmd);

        xhci::with_controller(|xhci| {
            // 1. Enviar CBW
            let slice_cbw =
                unsafe { core::slice::from_raw_parts_mut(&mut cbw as *mut _ as *mut u8, 31) };
            if !xhci.bulk_transfer(self.slot_id, self.ep_out, slice_cbw, false) {
                return false;
            }

            // 2. Data Phase
            if let Some(d) = data {
                if !xhci.bulk_transfer(
                    self.slot_id,
                    if is_in { self.ep_in } else { self.ep_out },
                    d,
                    is_in,
                ) {
                    return false;
                }
            }

            // 3. Status Phase (CSW)
            let mut csw = CommandStatusWrapper {
                signature: 0,
                tag: 0,
                data_residue: 0,
                status: 0,
            };
            let slice_csw =
                unsafe { core::slice::from_raw_parts_mut(&mut csw as *mut _ as *mut u8, 13) };
            if !xhci.bulk_transfer(self.slot_id, self.ep_in, slice_csw, true) {
                return false;
            }

            if csw.signature != 0x53425355 || csw.tag != current_tag {
                crate::core::debug::display::log(
                    "(USB-BOT) Erro: Assinatura ou Tag inválida no CSW.",
                );
                return false;
            }

            csw.status == 0
        })
        .unwrap_or(false)
    }
}
