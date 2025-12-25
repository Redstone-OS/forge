//! Interrupt Descriptor Table (IDT).
//!
//! Gerencia a tabela de interrupções.

use super::interrupts;
use core::arch::asm;
use core::mem::size_of;

/// Contexto salvo na stack durante uma interrupção.
#[repr(C)]
#[derive(Debug)]
pub struct ContextFrame {
    // Registradores salvos manualmente (pushall)
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rbp: u64,

    // Empilhado pela CPU ou pelo stub
    pub error_code: u64,

    // Empilhado pela CPU (Hardware Frame)
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    fn new(handler: usize) -> Self {
        Self {
            offset_low: handler as u16,
            selector: 0x08, // Kernel Code Segment
            ist: 0,
            type_attr: 0x8E, // Present | Ring0 | Interrupt Gate
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            zero: 0,
        }
    }

    fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }
}

#[repr(C, align(4096))]
struct Idt {
    entries: [IdtEntry; 256],
}

static mut IDT: Idt = Idt {
    entries: [IdtEntry {
        offset_low: 0,
        selector: 0,
        ist: 0,
        type_attr: 0,
        offset_mid: 0,
        offset_high: 0,
        zero: 0,
    }; 256],
};

#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base: u64,
}

/// Inicializa a IDT e registra os handlers básicos.
pub unsafe fn init() {
    // Limpar IDT (segurança)
    IDT.entries = [IdtEntry::missing(); 256];

    // Registrar Handlers Críticos
    IDT.entries[3] = IdtEntry::new(interrupts::breakpoint_handler as usize);
    IDT.entries[8] = IdtEntry::new(interrupts::double_fault_handler as usize);
    IDT.entries[13] = IdtEntry::new(interrupts::general_protection_fault_handler as usize);
    IDT.entries[14] = IdtEntry::new(interrupts::page_fault_handler as usize);

    // Timer (PIC remapeia IRQ0 para vetor 0x20 = 32)
    IDT.entries[32] = IdtEntry::new(interrupts::timer_handler as usize);

    // Carregar IDT
    let idt_ptr = IdtDescriptor {
        limit: (size_of::<Idt>() - 1) as u16,
        base: core::ptr::addr_of!(IDT) as u64,
    };

    asm!("lidt [{}]", in(reg) &idt_ptr, options(readonly, nostack, preserves_flags));

    // Interrupções ainda estão desabilitadas (CLI).
    // Só devem ser habilitadas (STI) após configurar o PIC/APIC.
}
