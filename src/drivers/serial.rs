// =============================================================================
// SERIAL DRIVER - ZERO OVERHEAD
// =============================================================================
//
// Driver de Porta Serial (COM1) para logging de kernel.
//
// ARQUITETURA:
// Este driver foi projetado para ser 100% livre de efeitos colaterais:
// - SEM Mutex/Spinlock - Escrita direta via I/O ports
// - SEM core::fmt - Evita geração de código SSE/AVX
// - SEM alocação - Apenas strings literais e valores imediatos
// - SEM interrupções - Não desabilita IRQs
//
// COMO FUNCIONA:
// Todas as funções usam assembly inline para enviar bytes diretamente
// para a porta serial COM1 (0x3F8). Isso garante:
// 1. Zero overhead de sincronização
// 2. Nenhuma instrução SIMD que possa causar #UD
// 3. Performance máxima em early-boot
//
// FUNÇÕES DISPONÍVEIS:
// - emit(byte)       : Envia um byte
// - emit_str(s)      : Envia string literal
// - emit_hex(v)      : Envia u64 em hexadecimal
// - emit_nl()        : Envia newline (\r\n)
//
// NOTA IMPORTANTE:
// Este driver NÃO garante exclusão mútua entre CPUs. Em ambiente SMP,
// os logs podem se intercalar. Isso é aceitável para debugging.
//
// =============================================================================

// Porta de dados da COM1
const COM1_DATA: u16 = 0x3F8;

// Porta de status da COM1 (Line Status Register)
const COM1_STATUS: u16 = 0x3FD;

// =============================================================================
// FUNÇÕES DE INICIALIZAÇÃO
// =============================================================================

/// Inicializa a porta serial COM1 (UART 16550).
///
/// Deve ser chamada uma vez durante o early-boot.
/// Configura: 38400 baud, 8N1, FIFO habilitado.
///
/// # Safety
/// Esta função acessa diretamente portas I/O.
pub fn init() {
    unsafe {
        // Disable interrupts
        port_out(COM1_DATA + 1, 0x00);

        // Enable DLAB (set baud rate divisor)
        port_out(COM1_DATA + 3, 0x80);

        // Set divisor to 3 (lo byte) = 38400 baud
        port_out(COM1_DATA, 0x03);

        // (hi byte)
        port_out(COM1_DATA + 1, 0x00);

        // 8 bits, no parity, one stop bit
        port_out(COM1_DATA + 3, 0x03);

        // Enable FIFO, clear them, with 14-byte threshold
        port_out(COM1_DATA + 2, 0xC7);

        // IRQs enabled, RTS/DSR set
        port_out(COM1_DATA + 4, 0x0B);
    }
}

// =============================================================================
// FUNÇÕES DE ESCRITA - CORE
// =============================================================================

/// Envia um único byte para a porta serial.
///
/// Esta é a função mais baixo nível. Todas as outras funções
/// de escrita usam esta internamente.
///
/// # Performance
/// - Usa assembly inline puro
/// - options(nostack, nomem, preserves_flags) = zero side effects
/// - Espera pelo buffer de transmissão estar livre (busy wait)
#[inline(always)]
pub fn emit(byte: u8) {
    unsafe {
        // Espera o buffer de transmissão estar vazio (bit 5 do LSR)
        loop {
            let status: u8;
            core::arch::asm!(
                "in al, dx",
                out("al") status,
                in("dx") COM1_STATUS,
                options(nostack, nomem, preserves_flags)
            );
            if (status & 0x20) != 0 {
                break;
            }
        }

        // Envia o byte
        core::arch::asm!(
            "out dx, al",
            in("al") byte,
            in("dx") COM1_DATA,
            options(nostack, nomem, preserves_flags)
        );
    }
}

/// Envia uma string para a porta serial.
///
/// # Uso
/// ```rust
/// serial::emit_str("Hello, World!");
/// ```
///
/// # Nota
/// Aceita apenas &str (strings). Para valores numéricos, use emit_hex().
#[inline(always)]
pub fn emit_str(s: &str) {
    for byte in s.bytes() {
        emit(byte);
    }
}

/// Envia uma nova linha (CRLF) para a porta serial.
///
/// Equivalente a: emit_str("\r\n")
#[inline(always)]
pub fn emit_nl() {
    emit(b'\r');
    emit(b'\n');
}

// =============================================================================
// FUNÇÕES DE ESCRITA - FORMATAÇÃO NUMÉRICA
// =============================================================================

/// Envia um valor u64 em formato hexadecimal.
///
/// Formato de saída: 0x0123456789ABCDEF (sempre 18 caracteres)
///
/// # Implementação
/// - Desenrolado manualmente para evitar loops/iteradores
/// - Conversão nibble->char inline (sem array lookup)
/// - Zero alocação
///
/// # Uso
/// ```rust
/// serial::emit_hex(0xDEADBEEF);  // Saída: 0x00000000DEADBEEF
/// ```
#[inline(always)]
pub fn emit_hex(value: u64) {
    // Prefixo "0x"
    emit(b'0');
    emit(b'x');

    // 16 nibbles (64 bits / 4 bits por nibble)
    // Desenrolado para evitar iteradores que podem gerar SSE
    emit(nibble_to_ascii(((value >> 60) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 56) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 52) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 48) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 44) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 40) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 36) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 32) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 28) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 24) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 20) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 16) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 12) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 8) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 4) & 0xF) as u8));
    emit(nibble_to_ascii((value & 0xF) as u8));
}

/// Envia um valor u32 em formato hexadecimal (versão compacta).
///
/// Formato de saída: 0x12345678 (sempre 10 caracteres)
#[inline(always)]
pub fn emit_hex32(value: u32) {
    emit(b'0');
    emit(b'x');
    emit(nibble_to_ascii(((value >> 28) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 24) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 20) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 16) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 12) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 8) & 0xF) as u8));
    emit(nibble_to_ascii(((value >> 4) & 0xF) as u8));
    emit(nibble_to_ascii((value & 0xF) as u8));
}

/// Envia um valor usize em formato decimal.
///
/// Útil para contadores e índices.
///
/// # Nota
/// Esta função usa um buffer de stack de 20 bytes (máximo para u64).
#[inline(always)]
pub fn emit_dec(mut value: usize) {
    // Buffer para dígitos (max 20 para u64)
    let mut buf: [u8; 20] = [0; 20];
    let mut pos = 20;

    if value == 0 {
        emit(b'0');
        return;
    }

    while value > 0 {
        pos -= 1;
        buf[pos] = b'0' + (value % 10) as u8;
        value /= 10;
    }

    while pos < 20 {
        emit(buf[pos]);
        pos += 1;
    }
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Converte nibble (0-15) para caractere ASCII ('0'-'9', 'A'-'F').
#[inline(always)]
const fn nibble_to_ascii(n: u8) -> u8 {
    if n < 10 {
        b'0' + n
    } else {
        b'A' + (n - 10)
    }
}

/// Escreve byte diretamente na porta I/O (helper interno).
#[inline(always)]
unsafe fn port_out(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("al") value,
        in("dx") port,
        options(nostack, nomem, preserves_flags)
    );
}
