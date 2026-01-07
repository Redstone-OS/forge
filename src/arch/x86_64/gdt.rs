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

/// Número máximo de CPUs suportadas
pub const MAX_CPUS: usize = 8;

// GDT global estática
// Entradas: Null, KCode, KData, UData, UCode, + (TSS-Low, TSS-High) * MAX_CPUS
// Index 5 + cpu*2 = TSS_Low da CPU, Index 6 + cpu*2 = TSS_High da CPU
const GDT_SIZE: usize = 5 + MAX_CPUS * 2; // 5 base + 2 por CPU

static mut GDT: [GdtEntry; GDT_SIZE] = {
    const NULL: GdtEntry = GdtEntry::null();
    [NULL; GDT_SIZE]
};

/// Inicializa entradas base da GDT (chamado uma vez pelo BSP)
unsafe fn init_gdt_base() {
    GDT[0] = GdtEntry::null();
    GDT[1] = GdtEntry::kernel_code();
    GDT[2] = GdtEntry::kernel_data();
    GDT[3] = GdtEntry::user_data(); // Antes de Code para SYSRET!
    GDT[4] = GdtEntry::user_code(); // SYSRET: CS = Base+16
                                    // Slots 5+ são para TSS de cada CPU (preenchidos em init_tss_for_cpu)
}

// Array de TSS per-CPU
static mut TSS_ARRAY: [Tss; MAX_CPUS] = [Tss::new(); MAX_CPUS];

// Stacks de Double Fault per-CPU (IST 1)
static mut DOUBLE_FAULT_STACKS: [[u8; 4096]; MAX_CPUS] = [[0; 4096]; MAX_CPUS];

/// Retorna o seletor TSS para uma CPU específica
pub fn tss_selector_for_cpu(cpu_id: usize) -> SegmentSelector {
    // TSS da CPU N está no index (5 + cpu_id * 2)
    let index = 5 + (cpu_id * 2) as u16;
    SegmentSelector::new(index, 0)
}

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
    // 1. Inicializar entradas base da GDT
    init_gdt_base();

    // 2. Configurar TSS para CPU 0 (BSP)
    init_tss_for_cpu(0);

    // 3. Carregar GDT
    let gdtr = GdtDescriptor {
        limit: (size_of::<[GdtEntry; GDT_SIZE]>() - 1) as u16,
        base: (&raw const GDT) as u64,
    };

    core::arch::asm!("lgdt [{}]", in(reg) &gdtr, options(readonly, nostack, preserves_flags));

    // 4. Recarregar Segmentos e TR
    let kcode = KERNEL_CODE_SEL.0;
    let kdata = KERNEL_DATA_SEL.0;
    let tss_sel = tss_selector_for_cpu(0).0;

    core::arch::asm!(
        "push {0:r}",           // Push CS (64-bit)
        "lea {1}, [rip + 2f]", // Load return address
        "push {1:r}",           // Push RIP
        "retfq",                // Far return to reload CS
        "2:",
        "mov ds, {2:x}",       // Reload DS
        "mov es, {2:x}",       // Reload ES
        "mov ss, {2:x}",       // Reload SS
        "mov ax, {3:x}",       // Load TSS selector
        "ltr ax",              // Load Task Register
        in(reg) kcode,
        out(reg) _,
        in(reg) kdata,
        in(reg) tss_sel,
        options(nostack)
    );
}

/// Configura o TSS de uma CPU específica na GDT
unsafe fn init_tss_for_cpu(cpu_id: usize) {
    if cpu_id >= MAX_CPUS {
        return;
    }

    let tss = &raw mut TSS_ARRAY[cpu_id];
    let tss_base = tss as u64;
    let tss_limit = (size_of::<Tss>() - 1) as u32;

    // Configurar IST 1 (Double Fault Stack) para esta CPU
    let df_stack_top = (&raw const DOUBLE_FAULT_STACKS[cpu_id] as u64) + 4096;
    (*tss).ist1 = df_stack_top;

    // Configurar descritores TSS na GDT (cada TSS usa 2 slots)
    let gdt_index = 5 + cpu_id * 2;
    GDT[gdt_index] = GdtEntry::tss_low(tss_base, tss_limit);
    GDT[gdt_index + 1] = GdtEntry::tss_high(tss_base);
}

/// Define o stack pointer do kernel (RSP0) no TSS da CPU atual
///
/// Usado pelo scheduler ao trocar de tasks.
pub unsafe fn set_kernel_stack(stack_top: u64) {
    let cpu_id = crate::sched::core::per_cpu::this_cpu_id();
    if cpu_id < MAX_CPUS {
        TSS_ARRAY[cpu_id].rsp0 = stack_top;
    }
}

/// Carrega a GDT do kernel em um AP (Application Processor) com TSS próprio
///
/// Os APs são inicializados com uma GDT temporária do trampoline.
/// Esta função carrega a GDT do kernel, configura o TSS desta CPU,
/// e recarrega os seletores de segmento incluindo o TR.
///
/// # Safety
///
/// Deve ser chamado durante inicialização do AP, antes de habilitar interrupções.
pub unsafe fn init_ap(cpu_id: usize) {
    // 1. Configurar TSS para esta CPU na GDT
    init_tss_for_cpu(cpu_id);

    // 2. Carregar GDT do kernel
    let gdtr = GdtDescriptor {
        limit: (size_of::<[GdtEntry; GDT_SIZE]>() - 1) as u16,
        base: (&raw const GDT) as u64,
    };

    core::arch::asm!("lgdt [{}]", in(reg) &gdtr, options(readonly, nostack, preserves_flags));

    // 3. Recarregar segmentos e carregar TSS desta CPU
    let kcode = KERNEL_CODE_SEL.0;
    let kdata = KERNEL_DATA_SEL.0;
    let tss_sel = tss_selector_for_cpu(cpu_id).0;

    core::arch::asm!(
        "push {0:r}",           // Push CS (64-bit)
        "lea {1}, [rip + 2f]", // Load return address
        "push {1:r}",           // Push RIP
        "retfq",                // Far return to reload CS
        "2:",
        "mov ds, {2:x}",       // Reload DS
        "mov es, {2:x}",       // Reload ES
        "mov ss, {2:x}",       // Reload SS
        "mov ax, {3:x}",       // Load TSS selector for this CPU
        "ltr ax",              // Load Task Register - cada CPU tem seu próprio TSS!
        in(reg) kcode,
        out(reg) _,
        in(reg) kdata,
        in(reg) tss_sel,
        options(nostack)
    );
}
