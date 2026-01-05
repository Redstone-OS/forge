// drivers/video/vbox_vga.rs
//
// Reescrita Rust do driver VBoxVga (Cirrus Logic 5430) para UEFI.
// Foco: Segurança de memória, uso de uefi-rs e remoção de macros inseguras.

#![no_std]
#![feature(abi_efiapi)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, ModeInfo, PixelFormat};
use uefi::proto::device_path::DevicePath;
use uefi::proto::driver_binding::DriverBinding;
use uefi::proto::pci::PciIo;
use uefi::table::boot::OpenProtocolAttributes;
use uefi::table::boot::OpenProtocolParams;
use uefi::{CStr16, Guid, Status};

// --- Definições de Hardware e Constantes ---

const VBOX_VENDOR_ID: u16 = 0x80EE;
const VBOX_VGA_DEVICE_ID: u16 = 0xBEEF; // Exemplo, verificar ID real

const CRTC_ADDRESS_REGISTER: u16 = 0x3d4;
const CRTC_DATA_REGISTER: u16 = 0x3d5;
const SEQ_ADDRESS_REGISTER: u16 = 0x3c4;
const SEQ_DATA_REGISTER: u16 = 0x3c5;
const GRAPH_ADDRESS_REGISTER: u16 = 0x3ce;
const GRAPH_DATA_REGISTER: u16 = 0x3cf;
const ATT_ADDRESS_REGISTER: u16 = 0x3c0;
const MISC_OUTPUT_REGISTER: u16 = 0x3c2;
const INPUT_STATUS_1_REGISTER: u16 = 0x3da;
const PALETTE_INDEX_REGISTER: u16 = 0x3c8;
const PALETTE_DATA_REGISTER: u16 = 0x3c9;

// --- Estruturas de Dados ---

#[repr(C)]
#[derive(Clone, Copy)]
struct VBoxVgaModeData {
    width: u32,
    height: u32,
    color_depth: u32,
    refresh_rate: u32,
    crtc_settings: &'static [u8; 25],
    seq_settings: &'static [u8; 5],
    misc_setting: u8,
}

// Definição dos modos (Imutáveis e Estáticos)
static CRTC_640_480: [u8; 25] = [
    0x5f, 0x4f, 0x50, 0x82, 0x54, 0x80, 0x0b, 0x3e, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xea, 0x0c, 0xdf, 0x28, 0x4f, 0xe7, 0x04, 0xe3, 0xff,
];
static SEQ_640_480: [u8; 5] = [0x01, 0x01, 0x0f, 0x00, 0x0a];

static MODES: &[VBoxVgaModeData] = &[
    VBoxVgaModeData {
        width: 640,
        height: 480,
        color_depth: 32,
        refresh_rate: 60,
        crtc_settings: &CRTC_640_480,
        seq_settings: &SEQ_640_480,
        misc_setting: 0xe3,
    },
    // Outros modos seriam adicionados aqui...
];

// --- Abstração de Hardware (I/O) ---

// Em UEFI x86, podemos usar instruções de porta diretamente ou via PciIo.
// O original usava macros ignorando o contexto PCI, então usaremos asm direto
// para fidelidade, mas encapsulado como unsafe.
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

unsafe fn outw(port: u16, val: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
}

unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    core::arch::asm!("in al, dx", out("al") ret, in("dx") port, options(nomem, nostack, preserves_flags));
    ret
}

// Helper para substituir a macro BOUTB
unsafe fn batch_outb(data: &[u8], addr_port: u16, data_port: u16) {
    for (i, &val) in data.iter().enumerate() {
        outb(addr_port, i as u8);
        outb(data_port, val);
    }
}

// --- Estrutura Privada do Driver ---

struct VBoxVgaPrivate {
    handle: Handle,
    pci_io: ScopedProtocol<PciIo>, // Wrapper RAII do uefi-rs seria ideal, aqui simplificado
    original_attributes: u64,
    current_mode: usize,
    // GOP e AppleFB seriam campos aqui se precisássemos manter estado
}

// --- Apple Framebuffer Protocol (Custom/Proprietário) ---

#[repr(C)]
struct AppleFramebufferInfoProtocol {
    get_info: extern "efiapi" fn(
        this: *const AppleFramebufferInfoProtocol,
        base_addr: *mut u32,
        reserved: *mut u32,
        row_bytes: *mut u32,
        width: *mut u32,
        height: *mut u32,
        depth: *mut u32,
    ) -> Status,
    private: *mut c_void, // Pointer to VBoxVgaPrivate
}

extern "efiapi" fn apple_fb_get_info(
    this: *const AppleFramebufferInfoProtocol,
    base_addr: *mut u32,
    _reserved: *mut u32,
    row_bytes: *mut u32,
    width: *mut u32,
    height: *mut u32,
    depth: *mut u32,
) -> Status {
    unsafe {
        let proto = &*this;
        let private = &*(proto.private as *const VBoxVgaPrivate);
        let mode = &MODES[private.current_mode];

        // Obter BAR 0 (Framebuffer)
        // Nota: PciIo em Rust requereria manipulação cuidadosa aqui.
        // Simplificado: private.pci_io.get_bar_attributes(...)

        // Simulação de retorno baseada no modo atual
        *width = mode.width;
        *height = mode.height;
        *depth = mode.color_depth;
        *row_bytes = (mode.width * mode.color_depth / 8);
        *base_addr = 0xE0000000; // Placeholder, viria do PCI BAR

        Status::SUCCESS
    }
}

// --- Lógica de Inicialização ---

fn initialize_graphics_mode(mode: &VBoxVgaModeData) {
    unsafe {
        // Sequência de inicialização VGA padrão
        outb(MISC_OUTPUT_REGISTER, mode.misc_setting);
        outb(SEQ_ADDRESS_REGISTER, 0);
        outb(SEQ_DATA_REGISTER, 1); // Reset Síncrono

        // Configuração genérica VBox
        outw(0x1ce, 0x00);
        outw(0x1cf, 0xb0c0); // Enable VBox Extensions
        outw(0x1ce, 0x01);
        outw(0x1cf, mode.width as u16);
        outw(0x1ce, 0x02);
        outw(0x1cf, mode.height as u16);
        outw(0x1ce, 0x03);
        outw(0x1cf, mode.color_depth as u16);

        // Aplica configurações de CRTC e Sequencer
        batch_outb(mode.seq_settings, SEQ_ADDRESS_REGISTER, SEQ_DATA_REGISTER);
        outb(SEQ_ADDRESS_REGISTER, 0);
        outb(SEQ_DATA_REGISTER, 3); // Fim do Reset

        batch_outb(
            mode.crtc_settings,
            CRTC_ADDRESS_REGISTER,
            CRTC_DATA_REGISTER,
        );

        // Ativação do Attribute Controller
        inb(INPUT_STATUS_1_REGISTER); // Reset flip-flop
        outb(ATT_ADDRESS_REGISTER, 0x20); // Enable video
    }
}

// --- Driver Binding Protocol Implementation ---

struct VBoxVgaDriver;

impl uefi::proto::driver_binding::DriverBindingProtocol for VBoxVgaDriver {
    fn supported(
        &self,
        controller: Handle,
        _child: Option<Handle>,
        _device_path: Option<&DevicePath>,
    ) -> Status {
        let bt = system_table().boot_services();

        // Tenta abrir o protocolo PCI I/O
        let pci_io = match bt.open_protocol::<PciIo>(
            OpenProtocolParams {
                handle: controller,
                agent: bt.image_handle(),
                controller: Some(controller),
            },
            OpenProtocolAttributes::GetProtocol,
        ) {
            Ok(p) => p,
            Err(_) => return Status::UNSUPPORTED,
        };

        let pci = unsafe { &*pci_io.interface.get() };

        // Lê VendorID/DeviceID (Offset 0)
        let mut pci_header = [0u32; 1];
        if pci
            .pci
            .read(
                uefi::proto::pci::EfiPciIoWidth::Uint32,
                0,
                1,
                pci_header.as_mut_ptr(),
            )
            .is_err()
        {
            return Status::UNSUPPORTED;
        }

        let vendor_id = (pci_header[0] & 0xFFFF) as u16;
        let device_id = (pci_header[0] >> 16) as u16;

        if vendor_id == VBOX_VENDOR_ID && device_id == VBOX_VGA_DEVICE_ID {
            Status::SUCCESS
        } else {
            Status::UNSUPPORTED
        }
    }

    fn start(&self, controller: Handle, _child: Option<Handle>) -> Status {
        let bt = system_table().boot_services();

        // 1. Abrir PCI I/O (ByDriver)
        let pci_io_scoped = match bt.open_protocol::<PciIo>(
            OpenProtocolParams {
                handle: controller,
                agent: bt.image_handle(),
                controller: Some(controller),
            },
            OpenProtocolAttributes::ByDriver,
        ) {
            Ok(p) => p,
            Err(e) => return e.status(),
        };

        let pci = unsafe { &mut *pci_io_scoped.interface.get() };

        // 2. Salvar e definir atributos PCI
        let original_attrs = pci
            .attributes(uefi::proto::pci::PciIoAttributeOperation::Get, 0, &mut 0)
            .unwrap_or(0);

        let status = pci.attributes(
            uefi::proto::pci::PciIoAttributeOperation::Enable,
            // IO | MEM | VGA_IO | VGA_MEM
            0x01 | 0x02 | 0x20 | 0x10,
            &mut 0,
        );
        if status.is_err() {
            return status.status();
        }

        // 3. Inicializar Hardware
        let default_mode = &MODES[0];
        initialize_graphics_mode(default_mode);

        // 4. Preparar estrutura privada
        let private = Box::new(VBoxVgaPrivate {
            handle: controller,
            pci_io: pci_io_scoped, // Ownership movido
            original_attributes: original_attrs,
            current_mode: 0,
        });

        let private_ptr = Box::into_raw(private);

        // 5. Instalar Protocolos (GOP, AppleFB)
        // Nota: Em Rust uefi, instalar protocolos envolve criar a interface na heap
        // e passar o ponteiro. Aqui simplificado para brevidade.

        let apple_fb = Box::new(AppleFramebufferInfoProtocol {
            get_info: apple_fb_get_info,
            private: private_ptr as *mut c_void,
        });

        // let guid = Guid::from_values(...);
        // bt.install_protocol_interface(controller, &guid, Box::into_raw(apple_fb) as *mut c_void);

        Status::SUCCESS
    }

    fn stop(
        &self,
        controller: Handle,
        _num_children: usize,
        _child_handle_buffer: Option<&[Handle]>,
    ) -> Status {
        let bt = system_table().boot_services();

        // 1. Recuperar estrutura privada (via OpenProtocol ou HandleDatabase)
        // Em um driver real, usariamos CRTP ou um mapa para recuperar o `Box<Private>`

        // 2. Restaurar atributos PCI
        // private.pci.attributes(Set, private.original_attributes);

        // 3. Desinstalar protocolos
        // bt.uninstall_protocol_interface(...)

        // 4. Fechar PCI I/O (Drop do ScopedProtocol faz isso automaticamente no Rust)

        Status::SUCCESS
    }
}

// --- Entry Point ---

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    // Instalar Driver Binding no Image Handle
    // let binding = VBoxVgaDriver;
    // ... lógica de instalação do driver binding ...

    Status::SUCCESS
}
