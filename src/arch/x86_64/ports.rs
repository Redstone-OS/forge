//! IO Ports (legado x86)

/// Lê um byte de uma porta IO
#[inline]
pub fn inb(port: u16) -> u8 {
    let value: u8;
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack)
        );
    }
    value
}

/// Escreve um byte em uma porta IO
#[inline]
pub fn outb(port: u16, value: u8) {
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack)
        );
    }
}

/// Lê um word (16 bits) de uma porta IO
#[inline]
pub fn inw(port: u16) -> u16 {
    let value: u16;
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "in ax, dx",
            in("dx") port,
            out("ax") value,
            options(nomem, nostack)
        );
    }
    value
}

/// Escreve um word em uma porta IO
#[inline]
pub fn outw(port: u16, value: u16) {
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") value,
            options(nomem, nostack)
        );
    }
}

/// Lê um dword (32 bits) de uma porta IO
#[inline]
pub fn inl(port: u16) -> u32 {
    let value: u32;
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "in eax, dx",
            in("dx") port,
            out("eax") value,
            options(nomem, nostack)
        );
    }
    value
}

/// Escreve um dword em uma porta IO
#[inline]
pub fn outl(port: u16, value: u32) {
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") value,
            options(nomem, nostack)
        );
    }
}

/// Delay de IO (espera ciclo de barramento)
#[inline]
pub fn io_wait() {
    // Porta 0x80 é usada para POST codes, escrever lá causa delay
    outb(0x80, 0);
}
