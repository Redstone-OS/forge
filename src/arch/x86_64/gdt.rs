//! Global Descriptor Table

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
pub const KERNEL_CODE_SEL: SegmentSelector = SegmentSelector::new(1, 0);
pub const KERNEL_DATA_SEL: SegmentSelector = SegmentSelector::new(2, 0);
pub const USER_CODE_SEL: SegmentSelector = SegmentSelector::new(3, 3);
pub const USER_DATA_SEL: SegmentSelector = SegmentSelector::new(4, 3);
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
            access: 0x9A,      // Present, Ring 0, Code, Readable
            flags_limit_high: 0xAF, // Long mode, limit high
            base_high: 0,
        }
    }
    
    pub const fn kernel_data() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0x92,      // Present, Ring 0, Data, Writable
            flags_limit_high: 0xCF,
            base_high: 0,
        }
    }
    
    pub const fn user_code() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0xFA,      // Present, Ring 3, Code, Readable
            flags_limit_high: 0xAF,
            base_high: 0,
        }
    }
    
    pub const fn user_data() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_mid: 0,
            access: 0xF2,      // Present, Ring 3, Data, Writable
            flags_limit_high: 0xCF,
            base_high: 0,
        }
    }
}

/// Task State Segment
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Tss {
    reserved0: u32,
    pub rsp0: u64,      // Stack para Ring 0
    pub rsp1: u64,
    pub rsp2: u64,
    reserved1: u64,
    pub ist1: u64,      // Interrupt Stack Table
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
static mut GDT: [GdtEntry; 7] = [
    GdtEntry::null(),
    GdtEntry::kernel_code(),
    GdtEntry::kernel_data(),
    GdtEntry::user_code(),
    GdtEntry::user_data(),
    GdtEntry::null(), // TSS low
    GdtEntry::null(), // TSS high
];

static mut TSS: Tss = Tss::new();

/// Inicializa a GDT
/// 
/// # Safety
/// 
/// Deve ser chamado apenas uma vez durante boot.
pub unsafe fn init() {
    // TODO: Configurar TSS entries na GDT
    // TODO: Carregar GDT com lgdt
    // TODO: Recarregar seletores de segmento
}
