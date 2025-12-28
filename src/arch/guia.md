# Guia de Implementação: `arch/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE. Não improvise. Não adicione nada que não esteja aqui.

---

## 1. PROPÓSITO DESTE MÓDULO

O módulo `arch/` é a **Hardware Abstraction Layer (HAL)**. Ele contém TODO o código específico de CPU. O resto do kernel NÃO SABE em qual arquitetura está rodando.

---

## 2. ESTRUTURA DE ARQUIVOS OBRIGATÓRIA

```
arch/
├── mod.rs              ✅ JÁ EXISTE - NÃO MODIFICAR
├── traits/
│   ├── mod.rs          → Exporta os traits
│   └── cpu.rs          → Trait CpuTrait
└── x86_64/
    ├── mod.rs          → Módulo raiz x86_64
    ├── cpu.rs          → Implementação CpuTrait
    ├── gdt.rs          → Global Descriptor Table
    ├── idt.rs          → Interrupt Descriptor Table
    ├── interrupts.rs   → Handlers de interrupção
    ├── memory.rs       → Setup inicial de paginação
    ├── ports.rs        → IO Ports (inb/outb)
    ├── syscall.rs      → Configuração SYSCALL/SYSRET
    ├── syscall.s       → Assembly do trampolim
    ├── switch.s        → Context switch assembly
    ├── acpi/
    │   ├── mod.rs
    │   ├── madt.rs     → Multiple APIC Description Table
    │   ├── fadt.rs     → Fixed ACPI Description Table
    │   └── dsdt.rs     → Differentiated System Description Table
    ├── apic/
    │   ├── mod.rs
    │   ├── lapic.rs    → Local APIC
    │   └── ioapic.rs   → I/O APIC
    └── iommu/
        ├── mod.rs
        └── intel_vtd.rs → Intel VT-d

    └── aarch64/
   
     Deve ter os arquivos mas como um todo para lembrar de implementacao futura

    └── riscv64/

    Deve ter os arquivos mas como um todo para lembrar de implementacao futura

```

---

## 3. REGRAS INVIOLÁVEIS

### ❌ NUNCA FAZER:
- Usar `f32` ou `f64` em qualquer lugar
- Usar `unwrap()` ou `expect()` fora de constantes
- Usar `std::` (estamos em `no_std`)
- Importar de outros módulos que não sejam `klib` ou `sync`
- Usar instruções SSE/AVX (compilador já desabilita, mas não tente forçar)

### ✅ SEMPRE FAZER:
- Comentário `// SAFETY:` antes de todo bloco `unsafe`
- Retornar `Result<T, Error>` ao invés de panic
- Documentar funções públicas com `///`
- Usar `#[repr(C)]` em structs que serão lidas por hardware

---

## 4. IMPLEMENTAÇÃO DETALHADA

### 4.1 `traits/mod.rs`

```rust
//! Traits abstratos para HAL

pub mod cpu;

pub use cpu::CpuTrait;
```

### 4.2 `traits/cpu.rs`

```rust
//! Trait abstrato para operações de CPU

/// Trait que toda arquitetura deve implementar
pub trait CpuTrait {
    /// Desabilita interrupções (cli)
    fn disable_interrupts();
    
    /// Habilita interrupções (sti)
    fn enable_interrupts();
    
    /// Para a CPU até próxima interrupção (hlt)
    fn halt();
    
    /// Retorna ID do core atual
    fn current_core_id() -> u32;
    
    /// Retorna se interrupções estão habilitadas
    fn interrupts_enabled() -> bool;
}
```

### 4.3 `x86_64/mod.rs`

```rust
//! Implementação x86_64

pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod memory;
pub mod ports;
pub mod syscall;

pub mod acpi;
pub mod apic;
pub mod iommu;

pub use cpu::Cpu;
```

### 4.4 `x86_64/cpu.rs`

```rust
//! Implementação de CPU para x86_64

use crate::arch::traits::CpuTrait;

/// Implementação x86_64 do trait CPU
pub struct Cpu;

impl CpuTrait for Cpu {
    #[inline(always)]
    fn disable_interrupts() {
        // SAFETY: cli é seguro, apenas desabilita interrupções
        unsafe { core::arch::asm!("cli", options(nomem, nostack)); }
    }
    
    #[inline(always)]
    fn enable_interrupts() {
        // SAFETY: sti é seguro, apenas habilita interrupções
        unsafe { core::arch::asm!("sti", options(nomem, nostack)); }
    }
    
    #[inline(always)]
    fn halt() {
        // SAFETY: hlt para CPU até próxima interrupção
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
    }
    
    fn current_core_id() -> u32 {
        // Lê APIC ID do LAPIC
        // TODO: Implementar leitura real do LAPIC
        0
    }
    
    fn interrupts_enabled() -> bool {
        let rflags: u64;
        // SAFETY: Leitura de RFLAGS é segura
        unsafe {
            core::arch::asm!(
                "pushfq",
                "pop {}",
                out(reg) rflags,
                options(nomem)
            );
        }
        (rflags & (1 << 9)) != 0 // Bit IF
    }
}

// Funções auxiliares que NÃO fazem parte do trait
impl Cpu {
    /// Lê um MSR (Model Specific Register)
    #[inline]
    pub fn read_msr(msr: u32) -> u64 {
        let (low, high): (u32, u32);
        // SAFETY: rdmsr lê MSR especificado em ECX
        unsafe {
            core::arch::asm!(
                "rdmsr",
                in("ecx") msr,
                out("eax") low,
                out("edx") high,
                options(nomem, nostack)
            );
        }
        ((high as u64) << 32) | (low as u64)
    }
    
    /// Escreve em um MSR
    #[inline]
    pub fn write_msr(msr: u32, value: u64) {
        let low = value as u32;
        let high = (value >> 32) as u32;
        // SAFETY: wrmsr escreve MSR especificado em ECX
        unsafe {
            core::arch::asm!(
                "wrmsr",
                in("ecx") msr,
                in("eax") low,
                in("edx") high,
                options(nomem, nostack)
            );
        }
    }
    
    /// Lê CR3 (Page Table Base)
    #[inline]
    pub fn read_cr3() -> u64 {
        let value: u64;
        // SAFETY: Leitura de CR3 é segura
        unsafe {
            core::arch::asm!("mov {}, cr3", out(reg) value, options(nomem, nostack));
        }
        value
    }
    
    /// Escreve CR3 (troca page table)
    /// 
    /// # Safety
    /// 
    /// O valor deve ser um endereço físico válido de uma page table.
    #[inline]
    pub unsafe fn write_cr3(value: u64) {
        // SAFETY: Caller garante que value é válido
        core::arch::asm!("mov cr3, {}", in(reg) value, options(nomem, nostack));
    }
}
```

### 4.5 `x86_64/gdt.rs`

```rust
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
```

### 4.6 `x86_64/ports.rs`

```rust
//! IO Ports (legado x86)

/// Lê um byte de uma porta IO
#[inline]
pub fn inb(port: u16) -> u8 {
    let value: u8;
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack)
        );
    }
    value
}

/// Escreve um byte em uma porta IO
#[inline]
pub fn outb(port: u16, value: u8) {
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack)
        );
    }
}

/// Lê um word (16 bits) de uma porta IO
#[inline]
pub fn inw(port: u16) -> u16 {
    let value: u16;
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "in ax, dx",
            in("dx") port,
            out("ax") value,
            options(nomem, nostack)
        );
    }
    value
}

/// Escreve um word em uma porta IO
#[inline]
pub fn outw(port: u16, value: u16) {
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") value,
            options(nomem, nostack)
        );
    }
}

/// Lê um dword (32 bits) de uma porta IO
#[inline]
pub fn inl(port: u16) -> u32 {
    let value: u32;
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "in eax, dx",
            in("dx") port,
            out("eax") value,
            options(nomem, nostack)
        );
    }
    value
}

/// Escreve um dword em uma porta IO
#[inline]
pub fn outl(port: u16, value: u32) {
    // SAFETY: IO ports são operações privilegiadas mas seguras
    unsafe {
        core::arch::asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") value,
            options(nomem, nostack)
        );
    }
}

/// Delay de IO (espera ciclo de barramento)
#[inline]
pub fn io_wait() {
    // Porta 0x80 é usada para POST codes, escrever lá causa delay
    outb(0x80, 0);
}
```

---

## 5. ORDEM DE IMPLEMENTAÇÃO

1. `traits/cpu.rs` - Definir contrato
2. `x86_64/ports.rs` - IO básico
3. `x86_64/cpu.rs` - Implementar trait
4. `x86_64/gdt.rs` - Segmentos
5. `x86_64/idt.rs` - Interrupções
6. `x86_64/interrupts.rs` - Handlers
7. `x86_64/syscall.rs` - SYSCALL/SYSRET
8. `x86_64/apic/` - LAPIC e IOAPIC
9. `x86_64/acpi/` - Parsing de tabelas
10. `x86_64/iommu/` - VT-d

---

## 6. TESTES OBRIGATÓRIOS

Criar arquivo `test.rs` com:

```rust
#[cfg(feature = "self_test")]
pub fn run_tests() {
    test_cpu_interrupts();
    test_io_ports();
}

fn test_cpu_interrupts() {
    use crate::arch::Cpu;
    use crate::arch::traits::CpuTrait;
    
    // Desabilitar e verificar
    Cpu::disable_interrupts();
    assert!(!Cpu::interrupts_enabled());
    
    // Habilitar e verificar
    Cpu::enable_interrupts();
    assert!(Cpu::interrupts_enabled());
    
    // Deixar desabilitado no final
    Cpu::disable_interrupts();
}

fn test_io_ports() {
    use crate::arch::x86_64::ports;
    
    // Testar porta 0x80 (POST codes, sempre funciona)
    ports::outb(0x80, 0xAA);
    // Não podemos ler de volta (write-only), mas não deve crashar
}
```

---

## 7. DEPENDÊNCIAS

Este módulo pode importar de:
- `crate::klib` (align, bitmap)
- `crate::sync` (spinlock apenas)

Este módulo NÃO pode importar de:
- `crate::mm`
- `crate::sched`
- `crate::ipc`
- `crate::fs`
- Qualquer outro módulo

---

## 8. CHECKLIST FINAL

- [ ] Todos os arquivos listados existem
- [ ] Nenhum `unwrap()` ou `expect()`
- [ ] Todo `unsafe` tem `// SAFETY:`
- [ ] Nenhum `f32`/`f64`
- [ ] Todas as funções públicas documentadas
- [ ] Testes existem e usam `#[cfg(feature = "self_test")]`
