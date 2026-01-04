//! # Controlador xHCI - Núcleo
//!
//! Gerenciamento de baixo nível, registros MMIO e inicialização de hardware.

use crate::drivers::pci::PciDevice;
use crate::mm::{translate_addr, VirtAddr};
use alloc::boxed::Box;
use alloc::vec::Vec;

use super::regs;
use super::ring::{EventRing, Ring};
use super::types::{UsbDevice, UsbPort, UsbSpeed};

/// Controlador xHCI
pub struct XhciController {
    pub(super) pci_device: PciDevice,
    pub(super) mmio_base: VirtAddr,
    pub(super) op_offset: u64,
    pub(super) runtime_offset: u64,
    pub(super) doorbell_offset: u64,
    pub(super) max_slots: u8,
    pub(super) max_ports: u8,
    pub(super) context_size: usize,
    pub(super) supports_64bit: bool,
    pub(super) command_ring: Option<Ring>,
    pub(super) event_ring: Option<EventRing>,
    pub(super) dcbaa_ptr: u64,
    pub(super) dcbaa_virt: u64,
    pub(super) devices: Vec<UsbDevice>,
    pub(super) mem_keepalive: Vec<Vec<u8>>,
    pub(super) initialized: bool,
}

unsafe impl Send for XhciController {}
unsafe impl Sync for XhciController {}

impl XhciController {
    pub fn new(pci_device: PciDevice) -> Option<Self> {
        pci_device.enable_bus_master();
        pci_device.enable_memory_space();

        let mmio_addr = pci_device.bar_address(0)?;
        let mut controller = Self {
            pci_device,
            mmio_base: VirtAddr::new(mmio_addr),
            op_offset: 0,
            runtime_offset: 0,
            doorbell_offset: 0,
            max_slots: 0,
            max_ports: 0,
            context_size: 32,
            supports_64bit: false,
            command_ring: None,
            event_ring: None,
            dcbaa_ptr: 0,
            dcbaa_virt: 0,
            devices: Vec::new(),
            mem_keepalive: Vec::new(),
            initialized: false,
        };

        if controller.init() {
            controller.initialized = true;
            Some(controller)
        } else {
            None
        }
    }

    pub(super) fn bios_handoff(&mut self, hcc1: u32) {
        let mut xecp = ((hcc1 >> 16) & 0xFFFF) << 2;
        if xecp == 0 {
            return;
        }

        unsafe {
            let base = self.mmio_base.as_u64();
            loop {
                let cap = core::ptr::read_volatile((base + xecp as u64) as *const u32);
                if (cap & 0xFF) == 1 {
                    // USB Legacy Support
                    // Seta OS Owned (bit 24)
                    core::ptr::write_volatile((base + xecp as u64) as *mut u32, cap | (1 << 24));

                    // Espera BIOS soltar (bit 16 deve ser 0) e OS assumir (bit 24 deve ser 1)
                    let mut timeout = 1000;
                    while timeout > 0 {
                        let val = core::ptr::read_volatile((base + xecp as u64) as *const u32);
                        if (val & (1 << 16)) == 0 && (val & (1 << 24)) != 0 {
                            break;
                        }
                        for _ in 0..1000 {
                            core::hint::spin_loop();
                        }
                        timeout -= 1;
                    }
                    if timeout == 0 {
                        crate::kwarn!("(xHCI) BIOS Handoff Timeout!");
                    }
                    break;
                }
                let next = (cap >> 8) & 0xFF;
                if next == 0 {
                    break;
                }
                xecp += (next << 2) as u32;
            }
        }
    }

    pub fn init(&mut self) -> bool {
        unsafe {
            // 1. Ler Capacidades Básicas
            let cap_len = self.read_cap32(0) & 0xFF;
            self.op_offset = cap_len as u64;

            let hcc1 = self.read_cap32(regs::cap::HCCPARAMS1);
            self.supports_64bit = (hcc1 & 0x1) != 0;
            self.context_size = if (hcc1 & 0x4) != 0 { 64 } else { 32 };

            let hcs1 = self.read_cap32(regs::cap::HCSPARAMS1);
            self.max_slots = (hcs1 & 0xFF) as u8;
            self.max_ports = ((hcs1 >> 24) & 0xFF) as u8;

            self.doorbell_offset = self.read_cap32(regs::cap::DBOFF) as u64;
            self.runtime_offset = self.read_cap32(regs::cap::RTSOFF) as u64;

            // 2. BIOS Handoff (Crucial para hardware real!)
            self.bios_handoff(hcc1);

            // 3. Reset do controlador
            if !self.halt() || !self.reset() {
                return false;
            }

            // 4. Configurar DCBAA (Obrigatório alinhar a 64 bytes)
            let dcbaa_len = (self.max_slots as usize + 1);
            let mut dcbaa_vec: Vec<u64> = Vec::with_capacity(dcbaa_len + 8);
            for _ in 0..dcbaa_vec.capacity() {
                dcbaa_vec.push(0);
            }
            let leaked_dcbaa = Box::leak(dcbaa_vec.into_boxed_slice());
            let dcbaa_raw_ptr = leaked_dcbaa.as_ptr() as u64;
            let aligned_dcbaa = (dcbaa_raw_ptr + 63) & !63;
            self.dcbaa_ptr = translate_addr(aligned_dcbaa).expect("DCBAA Phys");
            self.dcbaa_virt = aligned_dcbaa;

            // 5. SCRATCHPAD BUFFERS (Muitos notebooks exigem isso ou travam!)
            let hcs2 = self.read_cap32(regs::cap::HCSPARAMS2);
            let max_scratch = ((hcs2 >> 21) & 0x1F) | (((hcs2 >> 27) & 0x1F) << 5);
            if max_scratch > 0 {
                let mut scratch_array: Vec<u64> = Vec::with_capacity(max_scratch as usize);
                for _ in 0..max_scratch {
                    let page = Box::leak(Box::new([0u8; 4096])); // Aloca página real
                    let phys = translate_addr(page.as_ptr() as u64).unwrap();
                    scratch_array.push(phys);
                }
                let scratch_base = Box::leak(scratch_array.into_boxed_slice());
                let scratch_phys = translate_addr(scratch_base.as_ptr() as u64).unwrap();
                unsafe {
                    *(aligned_dcbaa as *mut u64) = scratch_phys; // Primeiro slot do DCBAA é para Scratchpad
                }
                self.mem_keepalive
                    .push(Vec::from(core::slice::from_raw_parts(
                        scratch_base.as_ptr() as *const u8,
                        max_scratch as usize * 8,
                    )));
            }
            self.write_op64(regs::op::DCBAAP, self.dcbaa_ptr);

            // 6. Command Ring (Alinhamento rigoroso)
            let cmd_ring = Ring::new_default().expect("Cmd Ring fail");
            let cmd_phys = cmd_ring.phys_addr().as_u64();
            self.write_op64(regs::op::CRCR, cmd_phys | 1);
            self.command_ring = Some(cmd_ring);

            // 7. Event Ring
            let evt_ring = EventRing::new(256).expect("Evt Ring fail");
            let erst_phys = evt_ring.erst_phys().as_u64();
            let int_off = self.runtime_offset + 0x20;
            self.write_mmio32((int_off + 0x08) as u32, 1);
            self.write_mmio32((int_off + 0x10) as u32, erst_phys as u32);
            self.write_mmio32((int_off + 0x14) as u32, (erst_phys >> 32) as u32);
            let erdp = evt_ring.dequeue_ptr();
            self.write_mmio32((int_off + 0x18) as u32, (erdp as u32) | 8);
            self.write_mmio32((int_off + 0x1C) as u32, (erdp >> 32) as u32);
            self.event_ring = Some(evt_ring);

            // 8. Ativar Slots e Rodar
            let mut config = self.read_op32(regs::op::CONFIG);
            self.write_op32(regs::op::CONFIG, (config & !0xFF) | (self.max_slots as u32));
            self.write_op32(regs::op::USBCMD, regs::usbcmd::RS);

            for _ in 0..10_000 {
                if (self.read_op32(regs::op::USBSTS) & regs::usbsts::HCH) == 0 {
                    return true;
                }
                core::hint::spin_loop();
            }
            false
        }
    }
    pub(super) fn halt(&mut self) -> bool {
        let cmd = self.read_op32(0);
        self.write_op32(0, cmd & !1);
        for _ in 0..100_000 {
            if (self.read_op32(4) & 1) != 0 {
                return true;
            }
        }
        false
    }

    pub(super) fn reset(&mut self) -> bool {
        self.write_op32(0, 2);
        for _ in 0..100_000 {
            if (self.read_op32(0) & 2) == 0 {
                return true;
            }
        }
        false
    }

    pub(super) fn read_cap32(&self, off: u64) -> u32 {
        unsafe { core::ptr::read_volatile((self.mmio_base.as_u64() + off) as *const u32) }
    }
    pub(super) fn read_op32(&self, off: u64) -> u32 {
        unsafe {
            core::ptr::read_volatile((self.mmio_base.as_u64() + self.op_offset + off) as *const u32)
        }
    }
    pub(super) fn write_op32(&mut self, off: u64, val: u32) {
        unsafe {
            core::ptr::write_volatile(
                (self.mmio_base.as_u64() + self.op_offset + off) as *mut u32,
                val,
            );
        }
    }
    pub fn read_port32(&self, port: u8, off: u64) -> u32 {
        self.read_op32(0x400 + (port as u64 - 1) * 0x10 + off)
    }
    pub fn write_port32(&mut self, port: u8, off: u64, val: u32) {
        self.write_op32(0x400 + (port as u64 - 1) * 0x10 + off, val);
    }
    pub(super) fn read_mmio32(&self, off: u32) -> u32 {
        unsafe { core::ptr::read_volatile((self.mmio_base.as_u64() + off as u64) as *const u32) }
    }
    pub(super) fn write_mmio32(&mut self, off: u32, val: u32) {
        unsafe {
            core::ptr::write_volatile((self.mmio_base.as_u64() + off as u64) as *mut u32, val);
        }
    }
    pub(super) fn update_erdp(&mut self, phys: u64) {
        let off = self.runtime_offset + 0x20 + 0x18;
        self.write_mmio32(off as u32, (phys as u32) | 8);
        self.write_mmio32((off + 4) as u32, (phys >> 32) as u32);
    }
    pub(super) fn ring_doorbell(&mut self, slot: u8, ep: u8) {
        let off = self.doorbell_offset + (slot as u64 * 4);
        unsafe {
            core::ptr::write_volatile((self.mmio_base.as_u64() + off) as *mut u32, ep as u32);
        }
    }
    pub(super) fn read_port_status(&self, port_num: u8) -> Option<UsbPort> {
        let status = self.read_port32(port_num, 0);
        Some(UsbPort {
            port_num,
            connected: (status & 1) != 0,
            enabled: (status & 2) != 0,
            speed: match (status >> 10) & 0xF {
                1 => UsbSpeed::Full,
                2 => UsbSpeed::Low,
                3 => UsbSpeed::High,
                4 => UsbSpeed::Super,
                5 => UsbSpeed::SuperPlus,
                _ => UsbSpeed::Full,
            },
        })
    }
    pub(super) fn write_op64(&mut self, off: u64, val: u64) {
        self.write_op32(off, val as u32);
        self.write_op32(off + 4, (val >> 32) as u32);
    }
}
