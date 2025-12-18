//! GDT (Global Descriptor Table)
//!
//! Implementa a GDT para x86_64 long mode.
//! Em long mode, segmentação é praticamente desabilitada, mas GDT ainda é necessária.

use core::mem::size_of;

/// Entrada da GDT
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    /// Cria nova entrada da GDT
    const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        Self {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: ((limit >> 16) & 0x0F) as u8 | (flags & 0xF0),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }

    /// Entrada nula (obrigatória na posição 0)
    const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }
}

/// Tabela GDT
#[repr(C, packed)]
struct Gdt {
    null: GdtEntry,
    kernel_code: GdtEntry,
    kernel_data: GdtEntry,
    user_code: GdtEntry,
    user_data: GdtEntry,
}

impl Gdt {
    /// Cria nova GDT com segmentos padrão
    const fn new() -> Self {
        // Flags de acesso
        const PRESENT: u8 = 1 << 7;
        const RING_0: u8 = 0 << 5;
        const RING_3: u8 = 3 << 5;
        const SYSTEM: u8 = 1 << 4;
        const EXECUTABLE: u8 = 1 << 3;
        const READWRITE: u8 = 1 << 1;

        // Flags de granularidade
        const LONG_MODE: u8 = 1 << 5;
        const SIZE_32: u8 = 1 << 6;
        const PAGE_GRANULAR: u8 = 1 << 7;

        Self {
            null: GdtEntry::null(),

            // Kernel code segment (0x08)
            kernel_code: GdtEntry::new(
                0,
                0,
                PRESENT | RING_0 | SYSTEM | EXECUTABLE | READWRITE,
                LONG_MODE,
            ),

            // Kernel data segment (0x10)
            kernel_data: GdtEntry::new(0, 0, PRESENT | RING_0 | SYSTEM | READWRITE, 0),

            // User code segment (0x18)
            user_code: GdtEntry::new(
                0,
                0,
                PRESENT | RING_3 | SYSTEM | EXECUTABLE | READWRITE,
                LONG_MODE,
            ),

            // User data segment (0x20)
            user_data: GdtEntry::new(0, 0, PRESENT | RING_3 | SYSTEM | READWRITE, 0),
        }
    }
}

/// Ponteiro para GDT (usado pelo LGDT)
#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

/// GDT global
static mut GDT: Gdt = Gdt::new();

/// Inicializa a GDT
pub fn init() {
    unsafe {
        let gdt_ptr = GdtPointer {
            limit: (size_of::<Gdt>() - 1) as u16,
            base: core::ptr::addr_of!(GDT) as u64,
        };

        // Carregar GDT
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &gdt_ptr,
            options(nostack, preserves_flags)
        );

        // Recarregar segmentos
        load_segments();
    }
}

/// Recarrega os registradores de segmento
unsafe fn load_segments() {
    // Recarregar segmentos de dados
    core::arch::asm!(
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ss, ax",
        out("ax") _,
    );

    // Recarregar CS com far return
    core::arch::asm!(
        "push 0x08",
        "lea {tmp}, [rip + 2f]",
        "push {tmp}",
        "retfq",
        "2:",
        tmp = lateout(reg) _,
    );
}
