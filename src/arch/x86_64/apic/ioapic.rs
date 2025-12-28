/// Arquivo: x86_64/apic/ioapic.rs
///
/// Propósito: Driver para o I/O APIC.
/// Responsável por rotear interrupções de hardware (teclado, disco, rede)
/// para um ou mais LAPICs (Cores). Substitui o PIC Master/Slave.
///
/// Detalhes de Implementação:
/// - Usa dois registradores mapeados em memória: IOREGSEL (Select) e IOWIN (Window).
/// - Endereço base padrão: 0xFEC00000.
/// - Suporta mapeamento de IRQs para vetores da IDT com destinos específicos (CPU ID).

//! Driver do I/O APIC

// Endereço Base Padrão
const IOAPIC_BASE_ADDR: u64 = 0xFEC00000;

// Offsets de Registradores (Memória)
const REG_IOREGSEL: usize = 0x00; // Selector Register
const REG_IOWIN: usize = 0x10;    // Window Register

// Índices de Registradores Internos (Acessados via Select/Window)
const IDX_ID: u32 = 0x00;
const IDX_VER: u32 = 0x01;
const IDX_ARB: u32 = 0x02;
const IDX_REDTBL_BASE: u32 = 0x10; // Redirection Tables (2 regs por entrada)

/// Inicializa o I/O APIC.
///
/// Por padrão, mascaramos todas as interrupções para evitar ruído.
/// Assume-se que o IOAPIC tem 24 entradas (padrão típico), mas lemos a versão para confirmar.
pub unsafe fn init() {
    // 1. Ler versão e quantidade de entradas
    let ver_val = read(IDX_VER);
    let max_entries = ((ver_val >> 16) & 0xFF) + 1; // Bits 16-23: Max Redir Entry

    // 2. Mascarar todas as entradas (Disable)
    for i in 0..max_entries {
        // Redirection Entry Layout:
        // Bit 16: Mask (1 = Masked)
        // Escrevemos Mask bit set e resto zero.
        write(IDX_REDTBL_BASE + 2 * i, 0x00010000); // Low 32 bits (Masked)
        write(IDX_REDTBL_BASE + 2 * i + 1, 0);      // High 32 bits (Dest)
    }
}

/// Mapeia uma IRQ (Interrupt Request) física para um vetor de interrupção e CPU.
///
/// # Argumentos
/// * `irq`: Número da IRQ (índice da entrada de redirecionamento, ex: 1 para Teclado).
/// * `vector`: Vetor na IDT (Ex: 33 para Teclado).
/// * `dest_lapic_id`: ID do LAPIC de destino.
pub unsafe fn map_irq(irq: u8, vector: u8, dest_lapic_id: u8) {
    let low_index = IDX_REDTBL_BASE + 2 * (irq as u32);
    let high_index = low_index + 1;

    // Configuração da Entrada:
    // Bits 0-7: Vector
    // Bits 8-10: Delivery Mode (000 = Fixed)
    // Bit 11: Destination Mode (0 = Physical)
    // Bit 16: Mask (0 = Unmasked)
    let low_val: u32 = vector as u32; // Fixed, Physical, Unmasked

    // Bits 56-63 (High 24-31): Destination (APIC ID)
    let high_val: u32 = (dest_lapic_id as u32) << 24;

    write(high_index, high_val);
    write(low_index, low_val);
}

/// Habilita uma IRQ específica
pub unsafe fn enable_irq(irq: u8) {
    let index = IDX_REDTBL_BASE + 2 * (irq as u32);
    let val = read(index);
    write(index, val & !(1 << 16)); // Clear Mask bit
}

/// Desabilita uma IRQ específica
pub unsafe fn disable_irq(irq: u8) {
    let index = IDX_REDTBL_BASE + 2 * (irq as u32);
    let val = read(index);
    write(index, val | (1 << 16)); // Set Mask bit
}

// --- Helpers de Acesso Indireto (Privados) ---

/// Lê um registrador de 32 bits do IOAPIC
unsafe fn read(reg_index: u32) -> u32 {
    let base = IOAPIC_BASE_ADDR as *mut u32;
    // 1. Escrever índice no IOREGSEL
    let ioregsel = base.add(REG_IOREGSEL / 4);
    core::ptr::write_volatile(ioregsel, reg_index);
    
    // 2. Ler valor do IOWIN
    let iowin = base.add(REG_IOWIN / 4);
    core::ptr::read_volatile(iowin)
}

/// Escreve em um registrador de 32 bits do IOAPIC
unsafe fn write(reg_index: u32, value: u32) {
    let base = IOAPIC_BASE_ADDR as *mut u32;
    // 1. Escrever índice no IOREGSEL
    let ioregsel = base.add(REG_IOREGSEL / 4);
    core::ptr::write_volatile(ioregsel, reg_index);
    
    // 2. Escrever valor no IOWIN
    let iowin = base.add(REG_IOWIN / 4);
    core::ptr::write_volatile(iowin, value);
}
