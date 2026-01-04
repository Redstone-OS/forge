//! # USB Mass Storage Driver
//!
//! Interface principal do driver de armazenamento USB.

pub mod scsi;
pub mod transport;

use crate::drivers::block::{BlockDevice, BlockError};
use crate::sync::Spinlock;
use alloc::sync::Arc;
use alloc::vec::Vec;
use scsi::{InquiryData, ScsiCommand};
use transport::BulkOnlyTransport;

pub struct UsbMassStorage {
    transport: BulkOnlyTransport,
    last_lba: Spinlock<u32>,
    ready: Spinlock<bool>,
}

impl UsbMassStorage {
    pub fn new(slot_id: u8, ep_in: u8, ep_out: u8, _vid: u16, _pid: u16) -> Self {
        Self {
            transport: BulkOnlyTransport::new(slot_id, ep_in, ep_out),
            last_lba: Spinlock::new(0),
            ready: Spinlock::new(false),
        }
    }

    pub fn initialize(&self) -> Result<(), BlockError> {
        crate::core::debug::display::log("(USB-MS) Iniciando descoberta...");

        // 1. INQUIRY - Identificar dispositivo
        let mut inq_buf = [0u8; 36];
        if self
            .transport
            .send_command(&ScsiCommand::inquiry(36), Some(&mut inq_buf), true)
        {
            let inq = unsafe { &*(inq_buf.as_ptr() as *const InquiryData) };
            let vendor = core::str::from_utf8(&inq.vendor_id).unwrap_or("Unknown");
            let product = core::str::from_utf8(&inq.product_id).unwrap_or("Unknown");
            crate::core::debug::display::log("(USB-MS) Device Identificado:");
            crate::core::debug::display::log(vendor);
            crate::core::debug::display::log(product);
        }

        // 2. TEST UNIT READY (com retry e Request Sense)
        let mut success = false;
        for _ in 0..5 {
            if self
                .transport
                .send_command(&ScsiCommand::test_unit_ready(), None, true)
            {
                success = true;
                break;
            } else {
                // Se falhar, pede o Sense para limpar o erro no hardware
                let mut sense_buf = [0u8; 18];
                let _ = self.transport.send_command(
                    &ScsiCommand::request_sense(18),
                    Some(&mut sense_buf),
                    true,
                );
            }
            // Pequeno delay para o hardware respirar
            for _ in 0..100_000 {
                core::hint::spin_loop();
            }
        }

        if !success {
            return Err(BlockError::NotReady);
        }

        // 3. READ CAPACITY
        let mut cap_data = [0u8; 8];
        if self
            .transport
            .send_command(&ScsiCommand::read_capacity_10(), Some(&mut cap_data), true)
        {
            let last_lba = u32::from_be_bytes([cap_data[0], cap_data[1], cap_data[2], cap_data[3]]);
            *self.last_lba.lock() = last_lba;
            *self.ready.lock() = true;
            crate::core::debug::display::log_hex("(USB-MS) Capacidade (LBA):", last_lba as u64);
            Ok(())
        } else {
            Err(BlockError::IoError)
        }
    }
}

impl BlockDevice for UsbMassStorage {
    fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), BlockError> {
        if !*self.ready.lock() {
            return Err(BlockError::NotReady);
        }
        let cmd = ScsiCommand::read_10(lba as u32, 1);
        if self.transport.send_command(&cmd, Some(buf), true) {
            Ok(())
        } else {
            // Tenta limpar erro se falhar leitura
            let mut sense_buf = [0u8; 18];
            let _ = self.transport.send_command(
                &ScsiCommand::request_sense(18),
                Some(&mut sense_buf),
                true,
            );
            Err(BlockError::IoError)
        }
    }

    fn write_block(&self, _lba: u64, _buf: &[u8]) -> Result<(), BlockError> {
        Err(BlockError::IoError)
    }
    fn block_size(&self) -> usize {
        512
    }
    fn total_blocks(&self) -> u64 {
        *self.last_lba.lock() as u64 + 1
    }
    fn is_read_only(&self) -> bool {
        false
    }
    fn flush(&self) -> Result<(), BlockError> {
        Ok(())
    }
}

static DEVICES: Spinlock<Vec<Arc<UsbMassStorage>>> = Spinlock::new(Vec::new());

pub fn register_device(device: Arc<UsbMassStorage>) {
    crate::drivers::block::register_device(device.clone());
    DEVICES.lock().push(device);
}
