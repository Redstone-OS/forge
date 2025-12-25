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
    crate::kdebug!("(IDT) init: Inicializando tabela de vetores...");

    // Limpar IDT (segurança)
    IDT.entries = [IdtEntry::missing(); 256];

    // Registrar Handlers Críticos
    IDT.entries[3] = IdtEntry::new(interrupts::breakpoint_handler as usize);
    IDT.entries[6] = IdtEntry::new(interrupts::invalid_opcode_handler as usize); // ADDED
    IDT.entries[8] = IdtEntry::new(interrupts::double_fault_handler as usize);
    IDT.entries[13] = IdtEntry::new(interrupts::general_protection_fault_handler as usize);
    IDT.entries[14] = IdtEntry::new(interrupts::page_fault_handler as usize);
    crate::ktrace!("(IDT) init: Exceções de CPU registradas");

    // Timer (PIC remapeia IRQ0 para vetor 0x20 = 32)
    IDT.entries[32] = IdtEntry::new(interrupts::timer_handler as usize);
    crate::ktrace!("(IDT) init: IRQ 0 (Timer) registrado");

    // Syscall API (Vector 0x80)
    extern "C" {
        fn syscall_handler();
    }
    let mut syscall_entry = IdtEntry::new(syscall_handler as usize);
    syscall_entry.type_attr = 0xEE; // Set DPL=3
    IDT.entries[0x80] = syscall_entry;
    crate::ktrace!("(IDT) init: Vetor 0x80 (Syscall) configurado");

    // Carregar IDT
    let idt_ptr = IdtDescriptor {
        limit: (size_of::<Idt>() - 1) as u16,
        base: core::ptr::addr_of!(IDT) as u64,
    };
    crate::ktrace!(
        "(IDT) init: IDTR base={:#x} limit={}",
        idt_ptr.base,
        idt_ptr.limit
    );

    asm!("lidt [{}]", in(reg) &idt_ptr, options(readonly, nostack, preserves_flags));

    crate::kinfo!("(IDT) Inicializado");
}
