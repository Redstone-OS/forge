//! # PIC Driver - 100% Assembly Implementation
//!
//! Driver do 8259 PIC (Programmable Interrupt Controller).
//!
//! ## GARANTIAS
//! - **Zero SSE/AVX**: Todo código é assembly inline puro
//! - **Determinístico**: Comportamento idêntico em todos os perfis de compilação
//!
//! ## Funções Disponíveis
//! - `init()` - Inicializa e remapeia PICs (IRQ 0-15 → Vectors 32-47)
//! - `send_eoi(irq)` - Envia End of Interrupt
//! - `mask(irq)` - Desabilita uma IRQ
//! - `unmask(irq)` - Habilita uma IRQ
//! - `set_masks(mask1, mask2)` - Define máscaras diretamente

/// Inicializa e remapeia os PICs.
///
/// Remapeia IRQ 0-7 para vectors 32-39 e IRQ 8-15 para vectors 40-47.
/// Isso evita conflito com exceções da CPU (vectors 0-31).
///
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn init() {
    unsafe {
        core::arch::asm!(
            // Salvar máscaras atuais
            "mov dx, 0x21",       // PIC1_DATA
            "in al, dx",
            "mov r8b, al",        // mask1 saved in r8

            "mov dx, 0xA1",       // PIC2_DATA
            "in al, dx",
            "mov r9b, al",        // mask2 saved in r9

            // ICW1: Initialize + ICW4 needed (0x11)
            "mov dx, 0x20",       // PIC1_CMD
            "mov al, 0x11",
            "out dx, al",

            "mov dx, 0xA0",       // PIC2_CMD
            "mov al, 0x11",
            "out dx, al",

            // ICW2: Vector offsets (32 for PIC1, 40 for PIC2)
            "mov dx, 0x21",       // PIC1_DATA
            "mov al, 32",
            "out dx, al",

            "mov dx, 0xA1",       // PIC2_DATA
            "mov al, 40",
            "out dx, al",

            // ICW3: Cascade identity
            "mov dx, 0x21",       // PIC1: IRQ2 has slave (0x04)
            "mov al, 4",
            "out dx, al",

            "mov dx, 0xA1",       // PIC2: Cascade identity = 2
            "mov al, 2",
            "out dx, al",

            // ICW4: 8086 mode (0x01)
            "mov dx, 0x21",
            "mov al, 1",
            "out dx, al",

            "mov dx, 0xA1",
            "mov al, 1",
            "out dx, al",

            // Restaurar máscaras
            "mov dx, 0x21",
            "mov al, r8b",
            "out dx, al",

            "mov dx, 0xA1",
            "mov al, r9b",
            "out dx, al",

            out("al") _,
            out("dx") _,
            out("r8") _,
            out("r9") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Envia End of Interrupt (EOI) para o PIC apropriado.
///
/// Deve ser chamado ao final de todo handler de IRQ.
/// Para IRQs 8-15, envia EOI para ambos PICs (slave primeiro).
///
/// # Arguments
/// * `irq` - Número da IRQ (0-15) ou vector (32-47)
///
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn send_eoi(irq: u8) {
    // Se for vector, converter para IRQ
    let irq_num = if irq >= 32 { irq - 32 } else { irq };

    unsafe {
        if irq_num >= 8 {
            // IRQ do slave (8-15): enviar EOI para PIC2 primeiro
            core::arch::asm!(
                "mov dx, 0xA0",   // PIC2_CMD
                "mov al, 0x20",   // EOI
                "out dx, al",
                out("al") _,
                out("dx") _,
                options(nostack, preserves_flags)
            );
        }

        // Sempre enviar EOI para PIC1 (master)
        core::arch::asm!(
            "mov dx, 0x20",   // PIC1_CMD
            "mov al, 0x20",   // EOI
            "out dx, al",
            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Desabilita (mask) uma IRQ específica.
///
/// # Arguments
/// * `irq` - Número da IRQ (0-15)
///
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn mask(irq: u8) {
    if irq >= 16 {
        return;
    }

    unsafe {
        if irq < 8 {
            // IRQ 0-7: PIC1 (port 0x21)
            let bit_mask: u8 = 1 << irq;
            core::arch::asm!(
                "mov dx, 0x21",
                "in al, dx",
                "or al, {mask}",
                "out dx, al",
                mask = in(reg_byte) bit_mask,
                out("al") _,
                out("dx") _,
                options(nostack, preserves_flags)
            );
        } else {
            // IRQ 8-15: PIC2 (port 0xA1)
            let bit_mask: u8 = 1 << (irq - 8);
            core::arch::asm!(
                "mov dx, 0xA1",
                "in al, dx",
                "or al, {mask}",
                "out dx, al",
                mask = in(reg_byte) bit_mask,
                out("al") _,
                out("dx") _,
                options(nostack, preserves_flags)
            );
        }
    }
}

/// Habilita (unmask) uma IRQ específica.
///
/// # Arguments
/// * `irq` - Número da IRQ (0-15)
///
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn unmask(irq: u8) {
    if irq >= 16 {
        return;
    }

    unsafe {
        if irq < 8 {
            // IRQ 0-7: PIC1 (port 0x21)
            let bit_mask: u8 = !(1 << irq);
            core::arch::asm!(
                "mov dx, 0x21",
                "in al, dx",
                "and al, {mask}",
                "out dx, al",
                mask = in(reg_byte) bit_mask,
                out("al") _,
                out("dx") _,
                options(nostack, preserves_flags)
            );
        } else {
            // IRQ 8-15: PIC2 (port 0xA1)
            let bit_mask: u8 = !(1 << (irq - 8));
            core::arch::asm!(
                "mov dx, 0xA1",
                "in al, dx",
                "and al, {mask}",
                "out dx, al",
                mask = in(reg_byte) bit_mask,
                out("al") _,
                out("dx") _,
                options(nostack, preserves_flags)
            );
        }
    }
}

/// Define as máscaras de ambos os PICs diretamente.
///
/// # Arguments
/// * `mask1` - Máscara do PIC1 (IRQ 0-7)
/// * `mask2` - Máscara do PIC2 (IRQ 8-15)
///
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn set_masks(mask1: u8, mask2: u8) {
    unsafe {
        core::arch::asm!(
            "mov dx, 0x21",
            "mov al, {m1}",
            "out dx, al",

            "mov dx, 0xA1",
            "mov al, {m2}",
            "out dx, al",

            m1 = in(reg_byte) mask1,
            m2 = in(reg_byte) mask2,
            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Lê as máscaras atuais dos PICs.
///
/// # Returns
/// Tupla (mask1, mask2) com as máscaras de PIC1 e PIC2.
///
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn get_masks() -> (u8, u8) {
    let mask1: u8;
    let mask2: u8;

    unsafe {
        core::arch::asm!(
            "mov dx, 0x21",
            "in al, dx",
            "mov {m1}, al",

            "mov dx, 0xA1",
            "in al, dx",
            "mov {m2}, al",

            m1 = out(reg_byte) mask1,
            m2 = out(reg_byte) mask2,
            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }

    (mask1, mask2)
}

/// Desabilita completamente o PIC (máscara todas as IRQs).
///
/// Útil quando migrando para APIC.
///
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn disable() {
    set_masks(0xFF, 0xFF);
}
