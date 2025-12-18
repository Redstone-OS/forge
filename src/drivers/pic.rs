//! PIC (Programmable Interrupt Controller)
//!
//! Driver para o 8259 PIC (controlador de interrupções programável).
//! Gerencia IRQs de hardware.

use core::arch::asm;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;

/// Escreve byte em porta I/O
#[inline]
unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nostack, preserves_flags));
}

/// Lê byte de porta I/O
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nostack, preserves_flags));
    value
}

/// Inicializa o PIC
pub fn init() {
    unsafe {
        // Salvar máscaras
        let mask1 = inb(PIC1_DATA);
        let mask2 = inb(PIC2_DATA);

        // Iniciar sequência de inicialização
        outb(PIC1_COMMAND, ICW1_INIT);
        outb(PIC2_COMMAND, ICW1_INIT);

        // ICW2: Offset dos vetores (32 para PIC1, 40 para PIC2)
        outb(PIC1_DATA, 32);
        outb(PIC2_DATA, 40);

        // ICW3: Configurar cascata (PIC2 no IRQ2 do PIC1)
        outb(PIC1_DATA, 4); // IRQ2 tem slave
        outb(PIC2_DATA, 2); // Cascade identity

        // ICW4: Modo 8086
        outb(PIC1_DATA, ICW4_8086);
        outb(PIC2_DATA, ICW4_8086);

        // Restaurar máscaras (todos desabilitados)
        outb(PIC1_DATA, 0xFF);
        outb(PIC2_DATA, 0xFF);
    }
}

/// Desmascara (habilita) um IRQ
pub fn unmask_irq(irq: u8) {
    unsafe {
        let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
        let value = inb(port) & !(1 << (irq % 8));
        outb(port, value);
    }
}

/// Mascara (desabilita) um IRQ
pub fn mask_irq(irq: u8) {
    unsafe {
        let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
        let value = inb(port) | (1 << (irq % 8));
        outb(port, value);
    }
}

/// Envia EOI (End of Interrupt) ao PIC
pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_COMMAND, 0x20);
        }
        outb(PIC1_COMMAND, 0x20);
    }
}
