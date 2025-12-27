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
/// # Implementação
/// 100% assembly - Zero SSE garantido.
#[inline(never)]
pub fn emit_str(s: &str) {
    let ptr = s.as_ptr();
    let len = s.len();

    unsafe {
        core::arch::asm!(
            // Loop de envio de bytes
            "2:",
            // Verificar se ainda há bytes para enviar
            "test {len}, {len}",
            "jz 4f",

            // Esperar UART pronto (busy wait no LSR bit 5)
            "3:",
            "mov dx, 0x3FD",        // COM1 + 5 = Line Status Register
            "in al, dx",
            "test al, 0x20",        // Bit 5 = transmit buffer empty
            "jz 3b",

            // Enviar byte
            "mov dx, 0x3F8",        // COM1 data port
            "mov al, [{ptr}]",      // Carregar byte da string
            "out dx, al",           // Enviar

            // Próximo byte
            "inc {ptr}",
            "dec {len}",
            "jmp 2b",

            "4:",
            ptr = inout(reg) ptr => _,
            len = inout(reg) len => _,
            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Envia uma nova linha (CRLF) para a porta serial.
///
/// # Implementação
/// 100% assembly - Zero SSE garantido.
#[inline(never)]
pub fn emit_nl() {
    unsafe {
        core::arch::asm!(
            // Enviar '\r' (0x0D)
            "2:",
            "mov dx, 0x3FD",
            "in al, dx",
            "test al, 0x20",
            "jz 2b",
            "mov dx, 0x3F8",
            "mov al, 0x0D",
            "out dx, al",

            // Enviar '\n' (0x0A)
            "3:",
            "mov dx, 0x3FD",
            "in al, dx",
            "test al, 0x20",
            "jz 3b",
            "mov dx, 0x3F8",
            "mov al, 0x0A",
            "out dx, al",

            out("al") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

// =============================================================================
// FUNÇÕES DE ESCRITA - FORMATAÇÃO NUMÉRICA
// =============================================================================

/// Envia um valor u64 em formato hexadecimal.
///
/// Formato de saída: 0x0123456789ABCDEF (sempre 18 caracteres)
///
/// # Implementação
/// 100% assembly - Zero SSE garantido.
#[inline(never)]
pub fn emit_hex(value: u64) {
    unsafe {
        core::arch::asm!(
            // Macro para enviar um byte (espera UART + out)
            // Usa r8 como scratch

            // Enviar '0'
            "2:",
            "mov dx, 0x3FD",
            "in al, dx",
            "test al, 0x20",
            "jz 2b",
            "mov dx, 0x3F8",
            "mov al, 0x30",    // '0'
            "out dx, al",

            // Enviar 'x'
            "3:",
            "mov dx, 0x3FD",
            "in al, dx",
            "test al, 0x20",
            "jz 3b",
            "mov dx, 0x3F8",
            "mov al, 0x78",    // 'x'
            "out dx, al",

            // Loop para 16 nibbles (começando do bit 60)
            "mov {shift}, 60",

            "4:",
            // Extrair nibble
            "mov {temp}, {val}",
            "mov cl, {shift:l}",
            "shr {temp}, cl",
            "and {temp}, 0xF",

            // Converter para ASCII
            "cmp {temp}, 10",
            "jb 5f",
            "add {temp}, 0x37",   // 'A' - 10 = 55 = 0x37
            "jmp 6f",
            "5:",
            "add {temp}, 0x30",   // '0' = 48 = 0x30
            "6:",

            // Esperar UART
            "7:",
            "mov dx, 0x3FD",
            "in al, dx",
            "test al, 0x20",
            "jz 7b",

            // Enviar nibble
            "mov dx, 0x3F8",
            "mov al, {temp:l}",
            "out dx, al",

            // Próximo nibble
            "sub {shift}, 4",
            "jns 4b",

            val = in(reg) value,
            shift = out(reg) _,
            temp = out(reg) _,
            out("al") _,
            out("cl") _,
            out("dx") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Envia um valor u32 em formato hexadecimal (versão compacta).
///
/// Formato de saída: 0x12345678 (sempre 10 caracteres)
#[inline(never)]
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
#[inline(never)]
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
