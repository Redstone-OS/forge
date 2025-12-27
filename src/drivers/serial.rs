//! # Serial Driver - 100% Assembly Implementation
//!
//! Driver de Porta Serial (COM1) para logging de kernel.
//!
//! ## GARANTIAS
//! - **Zero SSE/AVX**: Todo código é assembly inline puro
//! - **Zero Alocação**: Apenas valores imediatos e stack
//! - **Determinístico**: Comportamento idêntico em todos os perfis de compilação
//!
//! ## Funções Disponíveis
//! - `init()` - Inicializa COM1 (38400 baud, 8N1)
//! - `emit(byte)` - Envia um byte
//! - `emit_str(s)` - Envia string
//! - `emit_hex(v)` - Envia u64 em hex
//! - `emit_hex32(v)` - Envia u32 em hex
//! - `emit_dec(v)` - Envia usize em decimal
//! - `emit_nl()` - Envia CRLF

// Portas COM1
const COM1_DATA: u16 = 0x3F8;
const COM1_IER: u16 = 0x3F9; // Interrupt Enable Register
const COM1_FCR: u16 = 0x3FA; // FIFO Control Register
const COM1_LCR: u16 = 0x3FB; // Line Control Register
const COM1_MCR: u16 = 0x3FC; // Modem Control Register
const COM1_LSR: u16 = 0x3FD; // Line Status Register

/// Inicializa a porta serial COM1 (UART 16550).
///
/// Configura: 38400 baud, 8N1, FIFO habilitado.
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn init() {
    unsafe {
        core::arch::asm!(
            // Disable interrupts (IER = 0)
            "mov dx, {ier}",
            "xor al, al",
            "out dx, al",

            // Enable DLAB (LCR bit 7)
            "mov dx, {lcr}",
            "mov al, 0x80",
            "out dx, al",

            // Set divisor = 3 (38400 baud)
            // Low byte
            "mov dx, {data}",
            "mov al, 0x03",
            "out dx, al",

            // High byte
            "mov dx, {ier}",
            "xor al, al",
            "out dx, al",

            // 8 bits, no parity, one stop bit (LCR = 0x03)
            "mov dx, {lcr}",
            "mov al, 0x03",
            "out dx, al",

            // Enable FIFO, clear, 14-byte threshold (FCR = 0xC7)
            "mov dx, {fcr}",
            "mov al, 0xC7",
            "out dx, al",

            // IRQs enabled, RTS/DSR set (MCR = 0x0B)
            "mov dx, {mcr}",
            "mov al, 0x0B",
            "out dx, al",

            data = const COM1_DATA,
            ier = const COM1_IER,
            fcr = const COM1_FCR,
            lcr = const COM1_LCR,
            mcr = const COM1_MCR,
            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Envia um único byte para a porta serial.
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn emit(byte: u8) {
    unsafe {
        core::arch::asm!(
            // Wait for transmit buffer empty (LSR bit 5)
            "2:",
            "mov dx, {lsr}",
            "in al, dx",
            "test al, 0x20",
            "jz 2b",

            // Send byte
            "mov dx, {data}",
            "mov al, {byte}",
            "out dx, al",

            byte = in(reg_byte) byte,
            data = const COM1_DATA,
            lsr = const COM1_LSR,
            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Envia uma string para a porta serial.
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn emit_str(s: &str) {
    let ptr = s.as_ptr();
    let len = s.len();

    if len == 0 {
        return;
    }

    unsafe {
        core::arch::asm!(
            // String send loop
            "2:",
            // Check if bytes remain
            "test {len}, {len}",
            "jz 4f",

            // Wait for UART ready
            "3:",
            "mov dx, {lsr}",
            "in al, dx",
            "test al, 0x20",
            "jz 3b",

            // Send byte
            "mov dx, {data}",
            "mov al, [{ptr}]",
            "out dx, al",

            // Next byte
            "inc {ptr}",
            "dec {len}",
            "jmp 2b",

            "4:",
            ptr = inout(reg) ptr => _,
            len = inout(reg) len => _,
            data = const COM1_DATA,
            lsr = const COM1_LSR,
            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Envia CRLF para a porta serial.
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn emit_nl() {
    unsafe {
        core::arch::asm!(
            // Send '\r' (0x0D)
            "2:",
            "mov dx, {lsr}",
            "in al, dx",
            "test al, 0x20",
            "jz 2b",
            "mov dx, {data}",
            "mov al, 0x0D",
            "out dx, al",

            // Send '\n' (0x0A)
            "3:",
            "mov dx, {lsr}",
            "in al, dx",
            "test al, 0x20",
            "jz 3b",
            "mov dx, {data}",
            "mov al, 0x0A",
            "out dx, al",

            data = const COM1_DATA,
            lsr = const COM1_LSR,
            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Envia um valor u64 em formato hexadecimal (0x0123456789ABCDEF).
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn emit_hex(value: u64) {
    unsafe {
        core::arch::asm!(
            // Send '0'
            "2:",
            "mov dx, {lsr}",
            "in al, dx",
            "test al, 0x20",
            "jz 2b",
            "mov dx, {data}",
            "mov al, 0x30",
            "out dx, al",

            // Send 'x'
            "3:",
            "mov dx, {lsr}",
            "in al, dx",
            "test al, 0x20",
            "jz 3b",
            "mov dx, {data}",
            "mov al, 0x78",
            "out dx, al",

            // 16 nibbles loop (bit 60 down to 0)
            "mov {shift}, 60",

            "4:",
            // Extract nibble
            "mov {temp}, {val}",
            "mov cl, {shift:l}",
            "shr {temp}, cl",
            "and {temp}, 0xF",

            // Convert to ASCII
            "cmp {temp}, 10",
            "jb 5f",
            "add {temp}, 0x37",   // 'A' - 10
            "jmp 6f",
            "5:",
            "add {temp}, 0x30",   // '0'
            "6:",

            // Wait UART
            "7:",
            "mov dx, {lsr}",
            "in al, dx",
            "test al, 0x20",
            "jz 7b",

            // Send nibble
            "mov dx, {data}",
            "mov al, {temp:l}",
            "out dx, al",

            // Next nibble
            "sub {shift}, 4",
            "jns 4b",

            val = in(reg) value,
            data = const COM1_DATA,
            lsr = const COM1_LSR,
            shift = out(reg) _,
            temp = out(reg) _,
            out("al") _,
            out("cl") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Envia um valor u32 em formato hexadecimal (0x12345678).
/// 100% Assembly - Zero SSE.
#[inline(never)]
pub fn emit_hex32(value: u32) {
    unsafe {
        core::arch::asm!(
            // Send '0'
            "2:",
            "mov dx, {lsr}",
            "in al, dx",
            "test al, 0x20",
            "jz 2b",
            "mov dx, {data}",
            "mov al, 0x30",
            "out dx, al",

            // Send 'x'
            "3:",
            "mov dx, {lsr}",
            "in al, dx",
            "test al, 0x20",
            "jz 3b",
            "mov dx, {data}",
            "mov al, 0x78",
            "out dx, al",

            // 8 nibbles loop (bit 28 down to 0)
            "mov {shift:e}, 28",

            "4:",
            // Extract nibble
            "mov {temp:e}, {val:e}",
            "mov cl, {shift:l}",
            "shr {temp:e}, cl",
            "and {temp:e}, 0xF",

            // Convert to ASCII
            "cmp {temp:e}, 10",
            "jb 5f",
            "add {temp:e}, 0x37",
            "jmp 6f",
            "5:",
            "add {temp:e}, 0x30",
            "6:",

            // Wait UART
            "7:",
            "mov dx, {lsr}",
            "in al, dx",
            "test al, 0x20",
            "jz 7b",

            // Send nibble
            "mov dx, {data}",
            "mov al, {temp:l}",
            "out dx, al",

            // Next nibble
            "sub {shift:e}, 4",
            "jns 4b",

            val = in(reg) value,
            data = const COM1_DATA,
            lsr = const COM1_LSR,
            shift = out(reg) _,
            temp = out(reg) _,
            out("al") _,
            out("cl") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Envia um valor usize em formato decimal.
/// 100% Assembly - Zero SSE.
/// Usa buffer de 20 bytes na stack (máximo para u64).
#[inline(never)]
pub fn emit_dec(value: usize) {
    // Handle zero case specially
    if value == 0 {
        emit(b'0');
        return;
    }

    unsafe {
        // Stack buffer for digits (max 20 for u64)
        let mut buf: [u8; 20] = [0; 20];
        let buf_ptr = buf.as_mut_ptr();

        core::arch::asm!(
            // Convert to decimal (backwards)
            "mov {pos}, 20",
            "mov rax, {val}",

            "2:",
            // Divide by 10
            "xor rdx, rdx",
            "mov rcx, 10",
            "div rcx",           // rax = quotient, rdx = remainder

            // Store digit
            "dec {pos}",
            "add dl, 0x30",      // Convert to ASCII
            "mov [{buf} + {pos}], dl",

            // Continue if quotient != 0
            "test rax, rax",
            "jnz 2b",

            // Now output digits from pos to 20
            "3:",
            "cmp {pos}, 20",
            "jge 5f",

            // Wait UART
            "4:",
            "mov dx, {lsr}",
            "in al, dx",
            "test al, 0x20",
            "jz 4b",

            // Send digit
            "mov dx, {data}",
            "mov al, [{buf} + {pos}]",
            "out dx, al",

            "inc {pos}",
            "jmp 3b",

            "5:",

            val = in(reg) value,
            buf = in(reg) buf_ptr,
            pos = out(reg) _,
            data = const COM1_DATA,
            lsr = const COM1_LSR,
            out("rax") _,
            out("rcx") _,
            out("rdx") _,
            options(nostack, preserves_flags)
        );
    }
}
