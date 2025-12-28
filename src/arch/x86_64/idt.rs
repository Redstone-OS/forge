//! Interrupt Descriptor Table (IDT)

use core::mem::size_of;

/// Entrada da IDT (Gate Descriptor)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,        // Interrupt Stack Table offset
    flags: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    pub const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            flags: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }
    
    // TODO: Adicionar métodos para criar gates
}

/// A IDT contém 256 entradas
#[repr(C, align(16))]
pub struct Idt {
    pub entries: [IdtEntry; 256],
}

impl Idt {
    pub const fn new() -> Self {
        Self {
            entries: [IdtEntry::missing(); 256],
        }
    }
}

// IDT Global
static mut IDT: Idt = Idt::new();

/// Inicializa a IDT
///
/// # Safety
///
/// Deve ser chamado uma única vez no boot.
pub unsafe fn init() {
    // TODO: Configurar handlers de exceção
    // TODO: Carregar IDT com lidt
}
