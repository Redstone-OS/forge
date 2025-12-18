//! IDT (Interrupt Descriptor Table)
//!
//! Implementa a IDT para x86_64.
//! Mapeia vetores de interrupção (0-255) para handlers.

use core::mem::size_of;

/// Entrada da IDT
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,  // Handler address bits 0-15
    selector: u16,    // Code segment selector (0x08 = kernel code)
    ist: u8,          // Interrupt Stack Table (0 = não usa)
    type_attr: u8,    // Type and attributes
    offset_mid: u16,  // Handler address bits 16-31
    offset_high: u32, // Handler address bits 32-63
    reserved: u32,
}

impl IdtEntry {
    /// Cria nova entrada da IDT
    const fn new(handler: usize, selector: u16, ist: u8, type_attr: u8) -> Self {
        Self {
            offset_low: handler as u16,
            selector,
            ist,
            type_attr,
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            reserved: 0,
        }
    }

    /// Entrada vazia (não presente)
    const fn missing() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

/// Tabela IDT (256 entradas)
#[repr(C, align(16))]
struct Idt {
    entries: [IdtEntry; 256],
}

impl Idt {
    /// Cria nova IDT vazia
    const fn new() -> Self {
        Self {
            entries: [IdtEntry::missing(); 256],
        }
    }
}

/// Ponteiro para IDT (usado pelo LIDT)
#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

/// IDT global
static mut IDT: Idt = Idt::new();

/// Inicializa a IDT
pub fn init() {
    use super::interrupts::*;

    // Flags
    const PRESENT: u8 = 1 << 7;
    const RING_0: u8 = 0 << 5;
    const INTERRUPT_GATE: u8 = 0xE;
    const TYPE_ATTR: u8 = PRESENT | RING_0 | INTERRUPT_GATE;

    unsafe {
        // Registrar exception handlers (0-31)
        IDT.entries[0] = IdtEntry::new(divide_by_zero_handler as usize, 0x08, 0, TYPE_ATTR);
        IDT.entries[6] = IdtEntry::new(invalid_opcode_handler as usize, 0x08, 0, TYPE_ATTR);
        IDT.entries[13] = IdtEntry::new(
            general_protection_fault_handler as usize,
            0x08,
            0,
            TYPE_ATTR,
        );
        IDT.entries[14] = IdtEntry::new(page_fault_handler as usize, 0x08, 0, TYPE_ATTR);

        // Registrar IRQ handlers (32-47)
        IDT.entries[32] = IdtEntry::new(timer_interrupt_handler as usize, 0x08, 0, TYPE_ATTR);
        IDT.entries[33] = IdtEntry::new(keyboard_interrupt_handler as usize, 0x08, 0, TYPE_ATTR);

        // Registrar syscall handler (int 0x80 = 128)
        IDT.entries[0x80] = IdtEntry::new(syscall_interrupt_handler as usize, 0x08, 0, TYPE_ATTR);

        let idt_ptr = IdtPointer {
            limit: (size_of::<Idt>() - 1) as u16,
            base: core::ptr::addr_of!(IDT) as u64,
        };

        // Carregar IDT
        core::arch::asm!(
            "lidt [{}]",
            in(reg) &idt_ptr,
            options(nostack, preserves_flags)
        );
    }
}
