use super::controller::XhciController;
use super::regs;
use super::ring::Ring;
use super::structs::Trb;
use super::types::{UsbDevice, UsbSpeed};
use crate::drivers::usb::mass_storage;
use crate::mm::translate_addr;
use alloc::sync::Arc;
use alloc::vec::Vec;

impl XhciController {
    pub fn enable_slot(&mut self) -> Option<u8> {
        let mut trb = Trb::new();
        trb.set_type(regs::trb_type::ENABLE_SLOT);
        let resp = self.send_command(trb)?;
        Some((resp.control >> 24) as u8)
    }

    pub fn address_device(&mut self, slot_id: u8, port: u8, speed: UsbSpeed) -> bool {
        // 1. Alocar buffers para Input Context e Output Device Context (4KB alinhado)
        let mut input_vec: Vec<u8> = Vec::with_capacity(4096 + 64);
        for _ in 0..input_vec.capacity() {
            input_vec.push(0);
        }
        let input_ptr_raw = input_vec.as_ptr() as u64;
        let input_virt = (input_ptr_raw + 63) & !63;

        let mut output_vec: Vec<u8> = Vec::with_capacity(4096 + 64);
        for _ in 0..output_vec.capacity() {
            output_vec.push(0);
        }
        let output_ptr_raw = output_vec.as_ptr() as u64;
        let output_virt = (output_ptr_raw + 63) & !63;

        // 2. Registrar no DCBAA
        let slot_phys = translate_addr(output_virt).expect("Output Context Phys");
        unsafe {
            let entry = (self.dcbaa_virt + (slot_id as u64 * 8)) as *mut u64;
            entry.write_volatile(slot_phys);
        }

        // 3. ICC (Input Control Context) - Flags de adição A0 e A1
        unsafe {
            let add_flags = (input_virt + 4) as *mut u32;
            add_flags.write_volatile(0x03); // A0 (Slot) e A1 (EP0)
        }

        // 4. Slot Context (Offset 32)
        let slot_ctx = input_virt + 32;
        let speed_val = match speed {
            UsbSpeed::Low => 2,
            UsbSpeed::Full => 1,
            UsbSpeed::High => 3,
            UsbSpeed::Super => 4,
            _ => 1,
        };
        unsafe {
            let d0 = slot_ctx as *mut u32;
            d0.write_volatile((1 << 27) | (speed_val << 20)); // Context Entries = 1, Speed
            let d1 = (slot_ctx + 4) as *mut u32;
            d1.write_volatile((port as u32) << 16); // Port Num
        }

        // 5. EP0 Context (Offset 32 + context_size)
        let ep0_ctx = slot_ctx + self.context_size as u64;
        let ep0_ring = Ring::new_default().expect("EP0 Ring");
        let ep0_phys = ep0_ring.phys_addr().as_u64();

        let mps = match speed {
            UsbSpeed::Super => 512,
            UsbSpeed::High => 64,
            _ => 8,
        };

        unsafe {
            let d0 = ep0_ctx as *mut u32;
            d0.write_volatile(3 << 1); // CErr = 3
            let d1 = (ep0_ctx + 4) as *mut u32;
            d1.write_volatile((4 << 3) | (mps << 16)); // Type=Control, MPS
            let d2 = (ep0_ctx + 8) as *mut u32;
            d2.write_volatile((ep0_phys as u32) | 1); // DCS=1
            let d3 = (ep0_ctx + 12) as *mut u32;
            d3.write_volatile((ep0_phys >> 32) as u32);
        }

        // 6. Enviar comando ADDRESS_DEVICE
        let mut trb = Trb::new();
        trb.set_param_ptr(translate_addr(input_virt).expect("Input Context Phys"));
        trb.control = (slot_id as u32) << 24;
        trb.set_type(regs::trb_type::ADDRESS_DEVICE);

        if self.send_command(trb).is_some() {
            // Criar e salvar dispositivo
            let mut device = UsbDevice::new(slot_id, port, speed);
            device.rings[1] = Some(ep0_ring); // Salvar anel EP0
            self.devices.push(device);
            self.mem_keepalive.push(input_vec);
            self.mem_keepalive.push(output_vec);
            true
        } else {
            false
        }
    }

    pub fn enumerate_device(&mut self, slot_id: u8) {
        let device_index = self.devices.iter().position(|d| d.slot_id == slot_id);
        if let Some(index) = device_index {
            self.devices[index].device_class = 0x08; // Forçar Mass Storage para teste

            if self.devices[index].device_class == 0x08 {
                let driver = mass_storage::UsbMassStorage::new(slot_id, 1, 2, 0, 0);
                if driver.initialize().is_ok() {
                    mass_storage::register_device(Arc::new(driver));
                }
            }
        }
    }

    pub fn disable_slot(&mut self, slot_id: u8) -> bool {
        let mut trb = Trb::new();
        trb.control = (slot_id as u32) << 24;
        trb.set_type(regs::trb_type::DISABLE_SLOT);
        self.send_command(trb).is_some()
    }
}
