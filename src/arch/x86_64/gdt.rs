//! Global Descriptor Table (GDT) & Task State Segment (TSS).
//!
//! Configuração fundamental da arquitetura x86_64.
//!
//! # Responsabilidades
//! 1. **Segmentação (Legado obrigatório):** Define CS/DS para Kernel (Ring 0) e User (Ring 3).
//! 2. **TSS (Task State Segment):** Armazena o `RSP0` (Stack Pointer do Kernel).
//!    Quando uma interrupção ocorre em modo usuário, a CPU carrega automaticamente
//!    o `RSP0` do TSS para ter uma pilha segura onde salvar o contexto.
//!
//! # Segurança
//! - Estruturas `#[repr(C, packed)]` para garantir layout exato de hardware.
//! - Carregamento atômico de GDT e Segmentos via Assembly.

use core::arch::asm;
use core::mem::size_of;

// --- Seletores de Segmento (Exportados) ---
// Usados em `src/sched/task.rs` e `interrupts.rs`.
// O RPL (Requested Privilege Level) é codificado nos 2 bits inferiores (0 ou 3).

pub const KERNEL_CODE_SEL: u16 = 0x08;
pub const KERNEL_DATA_SEL: u16 = 0x10;
pub const USER_CODE_SEL: u16 = 0x18 | 3; // RPL 3
pub const USER_DATA_SEL: u16 = 0x20 | 3; // RPL 3
pub const TSS_SEL: u16 = 0x28;

// --- Estruturas de Hardware ---

/// Entrada padrão da GDT (64-bit Code/Data).
#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

/// Entrada de Segmento de Sistema (TSS é 16 bytes em 64-bit).
#[repr(C, packed)]
struct SystemSegmentEntry {
    low: GdtEntry,
    base_upper: u32,
    reserved: u32,
}

/// A Tabela GDT completa.
#[repr(C, align(4096))]
struct Gdt {
    null: GdtEntry,
    kernel_code: GdtEntry,
    kernel_data: GdtEntry,
    user_code: GdtEntry,
    user_data: GdtEntry,
    tss: SystemSegmentEntry,
}

/// Task State Segment (TSS) para x86_64.
#[repr(C, packed)]
struct Tss {
    reserved1: u32,
    rsp0: u64, // Stack do Kernel (Usado em interrupts de Ring 3)
    rsp1: u64,
    rsp2: u64,
    reserved2: u64,
    ist1: u64, // Interrupt Stack Tables (IST) - Opcional
    ist2: u64,
    ist3: u64,
    ist4: u64,
    ist5: u64,
    ist6: u64,
    ist7: u64,
    reserved3: u64,
    reserved4: u16,
    iomap_base: u16,
}

// --- Instâncias Globais Estáticas ---

static mut TSS: Tss = Tss {
    reserved1: 0,
    rsp0: 0,
    rsp1: 0,
    rsp2: 0,
    reserved2: 0,
    ist1: 0,
    ist2: 0,
    ist3: 0,
    ist4: 0,
    ist5: 0,
    ist6: 0,
    ist7: 0,
    reserved3: 0,
    reserved4: 0,
    iomap_base: 0xFFFF,
};

static mut GDT: Gdt = Gdt {
    null: GdtEntry::null(),
    // Ring 0 - Kernel
    kernel_code: GdtEntry::new(0x9A, 0x20), // Present, Ring0, Code, Exec/Read, LongMode
    kernel_data: GdtEntry::new(0x92, 0),    // Present, Ring0, Data, Read/Write
    // Ring 3 - User
    user_code: GdtEntry::new(0xFA, 0x20), // Present, Ring3, Code, Exec/Read, LongMode
    user_data: GdtEntry::new(0xF2, 0),    // Present, Ring3, Data, Read/Write
    // TSS (Preenchido dinamicamente no init)
    tss: SystemSegmentEntry::null(),
};

// --- Implementação ---

impl GdtEntry {
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

    const fn new(access: u8, flags: u8) -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access,
            granularity: flags,
            base_high: 0,
        }
    }
}

impl SystemSegmentEntry {
    const fn null() -> Self {
        Self {
            low: GdtEntry::null(),
            base_upper: 0,
            reserved: 0,
        }
    }

    /// Cria descritor TSS a partir do endereço da struct TSS.
    fn new_tss(tss_ptr: *const Tss) -> Self {
        let ptr = tss_ptr as u64;
        let size = size_of::<Tss>() as u64 - 1;

        Self {
            low: GdtEntry {
                limit_low: size as u16,
                base_low: ptr as u16,
                base_mid: (ptr >> 16) as u8,
                access: 0x89, // Present, Ring0, Available 64-bit TSS
                granularity: 0,
                base_high: (ptr >> 24) as u8,
            },
            base_upper: (ptr >> 32) as u32,
            reserved: 0,
        }
    }
}

/// Define a stack do Kernel para onde a CPU deve pular ao receber interrupção em Ring 3.
/// Deve ser chamada pelo Scheduler a cada troca de contexto.
///
/// # Safety
/// O endereço `stack_top` deve ser válido e mapeado.
pub unsafe fn set_kernel_stack(stack_top: u64) {
    TSS.rsp0 = stack_top;
}

#[repr(C, packed)]
struct GdtDescriptor {
    limit: u16,
    base: u64,
}

/// Inicializa a GDT e o TSS.
///
/// # Safety
/// Deve ser chamado apenas uma vez durante o boot do processador (BSP).
/// Em SMP, cada AP terá sua própria GDT/TSS.
pub unsafe fn init() {
    crate::ktrace!("(GDT) Configurando GDT e TSS...");

    // 1. Configurar entrada do TSS na GDT com o endereço real
    GDT.tss = SystemSegmentEntry::new_tss(core::ptr::addr_of!(TSS));
    crate::ktrace!("(GDT) TSS configurado na GDT");

    // 2. Carregar GDTR
    let gdt_ptr = GdtDescriptor {
        limit: (size_of::<Gdt>() - 1) as u16,
        base: core::ptr::addr_of!(GDT) as u64,
    };
    // Copiar campos packed para variáveis locais (evita E0793)
    let gdt_base = gdt_ptr.base;
    let gdt_limit = gdt_ptr.limit;
    crate::ktrace!("(GDT) GDTR base=", gdt_base);

    asm!("lgdt [{}]", in(reg) &gdt_ptr, options(readonly, nostack, preserves_flags));
    crate::ktrace!("(GDT) GDT carregada com sucesso");

    // 3. Recarregar Segmentos
    // CORREÇÃO E0658: Usamos registradores (in(reg)) para passar os seletores,
    // evitando a dependência de `const operands` instável.
    let kcode: u64 = KERNEL_CODE_SEL as u64;
    let kdata: u16 = KERNEL_DATA_SEL;
    let tss_sel: u16 = TSS_SEL;

    asm!(
        // Carregar segmentos de dados (DS, ES, FS, GS, SS) com KERNEL_DATA_SEL
        "mov ds, {dsel:x}",
        "mov es, {dsel:x}",
        "mov fs, {dsel:x}",
        "mov gs, {dsel:x}",
        "mov ss, {dsel:x}",

        // Carregar Code Segment (CS) usando far return hack
        "push {ksel}",      // Novo CS
        "lea {tmp}, [1f]",  // Endereço de retorno (Label 1)
        "push {tmp}",       // Empilha RIP
        "retfq",            // Far Return (pop RIP, pop CS)
        "1:",

        // Carregar Task Register (TR) com o seletor do TSS
        "ltr {tss_sel:x}",

        ksel = in(reg) kcode,
        dsel = in(reg) kdata,
        tss_sel = in(reg) tss_sel,
        tmp = lateout(reg) _,
    );

    crate::kinfo!("(GDT) Inicializada com sucesso");

    // NOTA: SSE desabilitado no target x86_64-redstone.json
    // init_sse() removido para evitar problemas com instruções SSE
}
