/// Arquivo: x86_64/gdt.rs
///
/// Propósito: Gerenciamento da Global Descriptor Table (GDT) e Task State Segment (TSS).
/// A GDT é usada para definir segmentos de memória (Código/Dados) para Kernel e Usuário.
/// O TSS é essencial para trocar de stacks durante interrupções (Interrupt Stack Table).
///
/// Detalhes de Implementação:
/// - Define seletores para Kernel Code/Data, User Code/Data e TSS.
/// - Inicializa a GDT estática e o TSS.
/// - Implementa o carregamento da GDT (`lgdt`) e recarregamento dos registradores de segmento.
/// - Configura a stack de interrupção no TSS (IST).
// Global Descriptor Table
use core::mem::size_of;

/// Seletor de segmento
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    pub const fn new(index: u16, rpl: u8) -> Self {
        Self((index << 3) | (rpl as u16))
    }
}

/// Constantes de seletores
// Index 0: Null
// Index 1: Kernel Code
// Index 2: Kernel Data
// Index 3: User Data  ← SYSRET requer Data antes de Code!
// Index 4: User Code  ← SYSRET: CS = Base+16, SS = Base+8
// Index 5: TSS (ocupa 2 slots em 64-bit)
pub const KERNEL_CODE_SEL: SegmentSelector = SegmentSelector::new(1, 0);
pub const KERNEL_DATA_SEL: SegmentSelector = SegmentSelector::new(2, 0);
pub const USER_DATA_SEL: SegmentSelector = SegmentSelector::new(3, 3); // Antes de Code para SYSRET!
pub const USER_CODE_SEL: SegmentSelector = SegmentSelector::new(4, 3); // Depois de Data para SYSRET!
pub const TSS_SEL: SegmentSelector = SegmentSelector::new(5, 0);

/// Entrada da GDT (64-bit)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    flags_limit_high: u8,
    base_high: u8,
}

impl GdtEntry {
    pub const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access: 0,
            flags_limit_high: 0,
            base_high: 0,
        }
    }

    pub const fn kernel_code() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0x9A,           // Present, Ring 0, Code, Readable
            flags_limit_high: 0xAF, // Long mode, limit high
            base_high: 0,
        }
    }

    pub const fn kernel_data() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0x92, // Present, Ring 0, Data, Writable
            flags_limit_high: 0xCF,
            base_high: 0,
        }
    }

    pub const fn user_code() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0xFA, // Present, Ring 3, Code, Readable
            flags_limit_high: 0xAF,
            base_high: 0,
        }
    }

    pub const fn user_data() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0xF2, // Present, Ring 3, Data, Writable
            flags_limit_high: 0xCF,
            base_high: 0,
        }
    }

    /// Cria descritor TSS (System Segment). TSS em 64-bit ocupa 16 bytes (2 entradas).
    /// Esta função cria a parte BAIXA.
    pub fn tss_low(base: u64, limit: u32) -> Self {
        Self {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_mid: ((base >> 16) & 0xFF) as u8,
            access: 0x89, // Present, Ring 0, Available TSS (0x9)
            flags_limit_high: (((limit >> 16) & 0xF) as u8) | 0x00, // Granularity 0
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }

    /// Cria a parte ALTA do descritor TSS.
    pub fn tss_high(base: u64) -> Self {
        Self {
            limit_low: ((base >> 32) & 0xFFFF) as u16,
            base_low: ((base >> 48) & 0xFFFF) as u16,
            base_mid: 0,
            access: 0,
            flags_limit_high: 0,
            base_high: 0,
        }
    }
}

/// Task State Segment (TSS)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Tss {
    reserved0: u32,
    pub rsp0: u64, // Stack para Ring 0 (usado em irq)
    pub rsp1: u64,
    pub rsp2: u64,
    reserved1: u64,
    pub ist1: u64, // Interrupt Stack Table
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    reserved2: u64,
    reserved3: u16,
    pub iomap_base: u16,
}

impl Tss {
    pub const fn new() -> Self {
        Self {
            reserved0: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            reserved1: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            reserved2: 0,
            reserved3: 0,
            iomap_base: size_of::<Tss>() as u16,
        }
    }
}

// GDT global estática
// 7 Entradas: Null, KCode, KData, UCode, UData, TSS-Low, TSS-High
static mut GDT: [GdtEntry; 7] = [
    GdtEntry::null(),
    GdtEntry::kernel_code(),
    GdtEntry::kernel_data(),
    GdtEntry::user_data(), // Index 3: User Data (antes de Code para SYSRET!)
    GdtEntry::user_code(), // Index 4: User Code (SYSRET: CS = Base+16)
    GdtEntry::null(),      // TSS low (será preenchido no init)
    GdtEntry::null(),      // TSS high (será preenchido no init)
];

// TSS global estática
static mut TSS: Tss = Tss::new();

/// Estrutura do Ponteiro da GDT (GDTR)
#[repr(C, packed)]
struct GdtDescriptor {
    limit: u16,
    base: u64,
}

/// Inicializa a GDT
///
/// # Safety
///
/// Deve ser chamado apenas uma vez durante boot (BSP).
/// Recarrega CS, DS, ES, SS, TR.
pub unsafe fn init() {
    // 1. Configurar entradas do TSS na GDT
    let tss_base = (&raw const TSS) as u64;
    let tss_limit = (size_of::<Tss>() - 1) as u32;

    GDT[5] = GdtEntry::tss_low(tss_base, tss_limit);
    GDT[6] = GdtEntry::tss_high(tss_base);

    // 2. Carregar GDT
    let gdtr = GdtDescriptor {
        limit: (size_of::<[GdtEntry; 7]>() - 1) as u16,
        base: (&raw const GDT) as u64,
    };

    core::arch::asm!("lgdt [{}]", in(reg) &gdtr, options(readonly, nostack, preserves_flags));

    // 3. Recarregar Segmentos
    // CS deve ser recarregado com um salto distante (retq hack) ou push/retq
    // DS, ES, SS devem ser carregados com KERNEL_DATA_SEL

    let kcode = KERNEL_CODE_SEL.0;
    let kdata = KERNEL_DATA_SEL.0;
    let tss_sel = TSS_SEL.0;

    core::arch::asm!(
        "push {0:r}",           // Push CS (64-bit)
        "lea {1}, [rip + 1f]", // Load return address (Intel syntax)
        "push {1}",           // Push RIP
        "retfq",                // Far return to reload CS
        "1:",
        "mov ds, {2:e}",      // Reload DS (32-bit reg name)
        "mov es, {2:e}",      // Reload ES
        "mov ss, {2:e}",      // Reload SS
        "mov ax, {3:x}",      // Load TSS selector (16-bit reg name)
        "ltr ax",             // Load Task Register
        in(reg) kcode,
        out(reg) _,
        in(reg) kdata,
        in(reg) tss_sel,
        options(nostack)
    );
}

/// Define o stack pointer do kernel (RSP0) no TSS
///
/// Usado pelo scheduler ao trocar de tasks.
pub unsafe fn set_kernel_stack(stack_top: u64) {
    TSS.rsp0 = stack_top;
}
