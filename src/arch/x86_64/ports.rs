/// Arquivo: x86_64/ports.rs
///
/// Propósito: Abstração para instruções de entrada/saída (I/O Ports) legadas do x86.
/// Port I/O primitives leitura e escrita em portas de I/O (inb, outb, etc.), essenciais para configurar
/// hardware legado como PIC, PIT, PS/2 e Serial, além de alguns registradores de PCI/DMA.
///
/// Detalhes de Implementação:
/// - Usa `core::arch::asm!` para emitir instruções `in` e `out`.
/// - Todas as funções são marcadas como `#[inline]` para evitar overhead.
/// - Implementa `io_wait` usando a porta 0x80 para garantir atrasos de barramento.

// IO Ports (legado x86)

/// Lê um byte de uma porta IO
#[inline]
pub fn inb(port: u16) -> u8 {
    let value: u8;
    // SAFETY: IO ports são operações privilegiadas mas seguras do ponto de vista de memória
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
    // SAFETY: IO ports são operações privilegiadas mas seguras do ponto de vista de memória
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
    // SAFETY: IO ports são operações privilegiadas mas seguras do ponto de vista de memória
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
    // SAFETY: IO ports são operações privilegiadas mas seguras do ponto de vista de memória
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
    // SAFETY: IO ports são operações privilegiadas mas seguras do ponto de vista de memória
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
    // SAFETY: IO ports são operações privilegiadas mas seguras do ponto de vista de memória
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
///
/// Usado quando o hardware precisa de um pequeno tempo para processar um comando
/// antes de receber o próximo (ex: PIC remapping).
#[inline]
pub fn io_wait() {
    // Porta 0x80 é usada para POST codes, escrever lá é seguro e causa um pequeno delay
    outb(0x80, 0);
}
