//! Driver do 8259 PIC (Programmable Interrupt Controller).
//!
//! Gerencia as interrupções de hardware (IRQs) antes de chegarem à CPU.
//! Em x86_64 moderno, o APIC é preferido, mas o PIC é necessário para o boot
//! ou como fallback.
//!
//! # Remapeamento
//! Por padrão, o PIC usa vetores 0-15, que conflitam com exceções da CPU.
//! Remapeamos para 32-47.

use crate::arch::x86_64::ports::Port;
use crate::sync::Mutex;

const PIC1_CMD: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const PIC_EOI: u8 = 0x20;

/// Cadeia de PICs (Master + Slave).
pub struct ChainedPics {
    pics: [Pic; 2],
}

struct Pic {
    offset: u8,
    command: Port<u8>,
    data: Port<u8>,
}

impl ChainedPics {
    pub const unsafe fn new(offset1: u8, offset2: u8) -> Self {
        Self {
            pics: [
                Pic {
                    offset: offset1,
                    command: Port::new(PIC1_CMD),
                    data: Port::new(PIC1_DATA),
                },
                Pic {
                    offset: offset2,
                    command: Port::new(PIC2_CMD),
                    data: Port::new(PIC2_DATA),
                },
            ],
        }
    }

    /// Inicializa e remapeia o PIC.
    pub unsafe fn init(&mut self) {
        crate::kdebug!("(PIC) init: Remapeando IRQs para vetores 32-47...");

        // Salvar máscaras
        let mask1 = self.pics[0].data.read();
        let mask2 = self.pics[1].data.read();
        crate::ktrace!("(PIC) init: Máscaras originais mask1=", mask1 as u64);
        crate::ktrace!("(PIC) init: Máscaras originais mask2=", mask2 as u64);

        // Sequência de inicialização (ICW1)
        self.pics[0].command.write(0x11);
        self.pics[1].command.write(0x11);
        crate::ktrace!("(PIC) init: ICW1 enviado");

        // ICW2: Offsets dos vetores
        self.pics[0].data.write(self.pics[0].offset);
        self.pics[1].data.write(self.pics[1].offset);
        crate::ktrace!("(PIC) init: ICW2 offset1=", self.pics[0].offset as u64);
        crate::ktrace!("(PIC) init: ICW2 offset2=", self.pics[1].offset as u64);

        // ICW3: Cascata
        self.pics[0].data.write(4); // IRQ2 tem slave
        self.pics[1].data.write(2); // Identidade cascade
        crate::ktrace!("(PIC) init: ICW3 cascata configurada");

        // ICW4: Modo 8086
        self.pics[0].data.write(0x01);
        self.pics[1].data.write(0x01);
        crate::ktrace!("(PIC) init: ICW4 modo 8086 pronto");

        // Restaurar máscaras
        self.pics[0].data.write(mask1);
        self.pics[1].data.write(mask2);
        crate::ktrace!("(PIC) init: Máscaras restauradas");

        crate::kinfo!("(PIC) Inicializado e Remapeado");
    }

    /// Envia "End of Interrupt" (EOI).
    /// Deve ser chamado ao final de todo handler de IRQ.
    pub unsafe fn notify_eoi(&mut self, interrupt_id: u8) {
        if interrupt_id >= self.pics[1].offset {
            self.pics[1].command.write(PIC_EOI);
        }
        self.pics[0].command.write(PIC_EOI);
    }

    /// Habilita (unmask) uma IRQ específica (0-15).
    pub unsafe fn unmask(&mut self, irq: u8) {
        let pic_idx = if irq < 8 { 0 } else { 1 };
        let port = &mut self.pics[pic_idx].data;
        let value = port.read();
        // Clear bit to enable
        port.write(value & !(1 << (irq % 8)));
    }
}

// Instância global protegida (Remapeando para 32 e 40)
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(32, 40) });
