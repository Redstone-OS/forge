//! Global Descriptor Table (GDT).
//!
//! Mesmo em 64-bit (Long Mode), a GDT é necessária para:
//! 1. Definir segmentos de Código/Dados (Kernel vs User).
//! 2. Carregar o TSS (Task State Segment) para troca de stacks em interrupções.

use core::arch::asm;
use core::mem::size_of;

/// Estrutura de entrada da GDT (64-bit friendly).
#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    /// Cria uma entrada nula (obrigatória).
    const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }

    /// Cria um segmento de código/dados padrão para 64-bit.
    const fn new(access: u8, flags: u8) -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access,
            granularity: flags, // Em 64-bit, limites são ignorados para a maioria dos segmentos
            base_high: 0,
        }
    }
}

#[repr(C, align(4096))] // Alinhamento de página é boa prática
struct Gdt {
    null: GdtEntry,
    kernel_code: GdtEntry,
    kernel_data: GdtEntry,
    user_code: GdtEntry,
    user_data: GdtEntry,
}

// Flags de Acesso
const ACCESS_PRESENT: u8 = 0x80;
const ACCESS_DESCRIPTOR: u8 = 0x10; // 1 = Código/Dados, 0 = Sistema
const ACCESS_EXECUTABLE: u8 = 0x08;
const ACCESS_RW: u8 = 0x02; // Leitura para código, Escrita para dados
const ACCESS_PRIV_KERNEL: u8 = 0x00;
const ACCESS_PRIV_USER: u8 = 0x60;

// Flags de Granularidade
const FLAG_LONG_MODE: u8 = 0x20;

static mut GDT: Gdt = Gdt {
    null: GdtEntry::null(),
    // Offset 0x08: Kernel Code
    kernel_code: GdtEntry::new(
        ACCESS_PRESENT | ACCESS_DESCRIPTOR | ACCESS_EXECUTABLE | ACCESS_RW | ACCESS_PRIV_KERNEL,
        FLAG_LONG_MODE,
    ),
    // Offset 0x10: Kernel Data
    kernel_data: GdtEntry::new(
        ACCESS_PRESENT | ACCESS_DESCRIPTOR | ACCESS_RW | ACCESS_PRIV_KERNEL,
        0,
    ),
    // Offset 0x18: User Code
    user_code: GdtEntry::new(
        ACCESS_PRESENT | ACCESS_DESCRIPTOR | ACCESS_EXECUTABLE | ACCESS_RW | ACCESS_PRIV_USER,
        FLAG_LONG_MODE,
    ),
    // Offset 0x20: User Data
    user_data: GdtEntry::new(
        ACCESS_PRESENT | ACCESS_DESCRIPTOR | ACCESS_RW | ACCESS_PRIV_USER,
        0,
    ),
};

#[repr(C, packed)]
struct GdtDescriptor {
    limit: u16,
    base: u64,
}

/// Carrega a GDT e recarrega os registradores de segmento.
///
/// # Safety
/// Mexe com estado global da CPU. Deve ser chamado apenas uma vez no boot.
pub unsafe fn init() {
    let gdt_ptr = GdtDescriptor {
        limit: (size_of::<Gdt>() - 1) as u16,
        base: core::ptr::addr_of!(GDT) as u64,
    };

    asm!("lgdt [{}]", in(reg) &gdt_ptr, options(readonly, nostack, preserves_flags));

    // Recarregar Segmentos
    // CS (Code Segment) precisa de um 'far jump' ou 'retfq'.
    // DS, ES, FS, GS, SS recebem o seletor de dados do Kernel (0x10).
    asm!(
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ss, ax",
        "push 0x08",        // Novo CS
        "lea {tmp}, [1f]",  // Endereço de retorno
        "push {tmp}",
        "retfq",            // Far return (simula far jump)
        "1:",
        tmp = lateout(reg) _,
    );
}
