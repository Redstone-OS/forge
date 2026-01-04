//! # Funções de I/O para ATA
//!
//! Wrappers para instruções x86 de I/O de porta.

use core::arch::asm;

/// Lê um byte de uma porta I/O
#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    value
}

/// Escreve um byte em uma porta I/O
#[inline]
pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}

/// Lê uma word (16 bits) de uma porta I/O
#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack));
    value
}

/// Escreve uma word (16 bits) em uma porta I/O
#[inline]
#[allow(dead_code)]
pub unsafe fn outw(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack));
}

/// Lê uma dword (32 bits) de uma porta I/O
#[inline]
#[allow(dead_code)]
pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack));
    value
}

/// Escreve uma dword (32 bits) em uma porta I/O
#[inline]
#[allow(dead_code)]
pub unsafe fn outl(port: u16, value: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack));
}

/// Lê múltiplas words de uma porta I/O usando REP INSW
///
/// # Safety
/// - O buffer deve ter espaço suficiente para `count` words (count * 2 bytes)
#[inline]
#[allow(dead_code)]
pub unsafe fn insw(port: u16, buf: *mut u16, count: usize) {
    asm!(
        "rep insw",
        in("dx") port,
        in("rdi") buf,
        in("rcx") count,
        options(nostack)
    );
}

/// Escreve múltiplas words em uma porta I/O usando REP OUTSW
///
/// # Safety
/// - O buffer deve ter pelo menos `count` words (count * 2 bytes)
#[inline]
#[allow(dead_code)]
pub unsafe fn outsw(port: u16, buf: *const u16, count: usize) {
    asm!(
        "rep outsw",
        in("dx") port,
        in("rsi") buf,
        in("rcx") count,
        options(nostack)
    );
}

/// Delay de 400ns usando 4 leituras do alternate status register
///
/// Necessário após certas operações ATA para dar tempo ao drive processar
#[inline]
pub fn io_delay() {
    unsafe {
        // Ler alternate status 4 vezes (~400ns delay)
        for _ in 0..4 {
            let _ = inb(0x3F6);
        }
    }
}
