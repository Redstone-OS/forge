/// Arquivo: x86_64/idt.rs
///
/// Propósito: Gerenciamento da Interrupt Descriptor Table (IDT).
/// Define a estrutura da tabela usada pela CPU para despachar exceções de hardware (Page Fault, GPF, etc.)
/// e interrupções externas (Irqs).
///
/// Detalhes de Implementação:
/// - Define `IdtEntry` conforme especificação AMD64/Intel 64.
/// - Mantém uma tabela estática de 256 entradas.
/// - Fornece métodos para registrar handlers.
/// - Implementa `load` para configurar o registrador IDTR.
// Interrupt Descriptor Table
use crate::arch::x86_64::gdt::KERNEL_CODE_SEL;
use core::mem::size_of;

// Alias ContextFrame to TrapFrame for syscall handling compatibility
pub use crate::arch::x86_64::syscall::TrapFrame as ContextFrame;

/// Tipo de função para Handler de Interrupção
/// Um handler "cru" recebe o ponteiro da stack se usarmos trampolins assembly,
/// ou pode ser uma função `extern "x86-interrupt"` se usarmos recursos nightly do Rust.
/// Por compatibilidade e controle, assumimos endereços `u64` (ponteiros para funções assembly).
pub type HandlerFunc = u64;

/// Entrada da IDT (16 bytes em 64-bit)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist_reserved_legacy: u8, // Bits 0-2: IST, 3-7: Reservado
    type_attr: u8,           // Gate Type, DPL, Present
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    /// Cria uma entrada vazia (não presente)
    pub const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist_reserved_legacy: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    /// Cria uma entrada presente apontando para um handler
    ///
    /// `ist`: Index da Interrupt Stack Table (1-7) no TSS. 0 para não usar.
    pub fn new(handler: HandlerFunc, ist: u8) -> Self {
        let addr = handler;
        Self {
            offset_low: (addr & 0xFFFF) as u16,
            selector: KERNEL_CODE_SEL.0,
            ist_reserved_legacy: ist & 0x7, // Apenas 3 bits para IST
            type_attr: 0x8E,                // Present, DPL 0, Interrupt Gate
            offset_mid: ((addr >> 16) & 0xFFFF) as u16,
            offset_high: (addr >> 32) as u32,
            reserved: 0,
        }
    }
}

/// A Tabela IDT propriamente dita
#[repr(C, align(16))]
pub struct Idt {
    entries: [IdtEntry; 256],
}

impl Idt {
    pub const fn new() -> Self {
        Self {
            entries: [IdtEntry::missing(); 256],
        }
    }

    /// Define um handler para o índice vector
    pub fn set_handler(&mut self, vector: u8, handler: HandlerFunc) {
        self.entries[vector as usize] = IdtEntry::new(handler, 0);
    }

    /// Define um handler usando uma Stack IST específica
    pub fn set_handler_ist(&mut self, vector: u8, handler: HandlerFunc, ist_index: u8) {
        self.entries[vector as usize] = IdtEntry::new(handler, ist_index);
    }

    /// Carrega a IDT na CPU (lidt)
    ///
    /// # Safety
    ///
    /// `lidt` é unsafe. A tabela deve ter tempo de vida 'static ou ser válida enquanto usada.
    pub unsafe fn load(&'static self) {
        let descriptor = IdtDescriptor {
            limit: (size_of::<Self>() - 1) as u16,
            base: (self as *const Self) as u64,
        };
        core::arch::asm!("lidt [{}]", in(reg) &descriptor, options(readonly, nostack, preserves_flags));
    }
}

/// Descritor para LIDT
#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base: u64,
}

// Global IDT (estática e mutável apenas na init)
pub static mut IDT: Idt = Idt::new();
