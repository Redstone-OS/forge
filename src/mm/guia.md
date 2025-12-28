# Guia de Implementação: `mm/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE.

---

## 1. PROPÓSITO

Gerenciamento completo de memória: física (PMM), virtual (VMM), e heap.

---

## 2. ESTRUTURA

```
mm/
├── mod.rs              ✅ JÁ EXISTE
├── config.rs           → Constantes e configuração
├── error.rs            → Tipos de erro
├── oom.rs              → Out-of-memory handler
├── addr/
│   ├── mod.rs
│   ├── phys.rs         → PhysAddr
│   ├── virt.rs         → VirtAddr
│   └── translate.rs    → Conversões
├── pmm/
│   ├── mod.rs
│   ├── frame.rs        → PhysFrame
│   ├── bitmap.rs       → Bitmap de frames
│   ├── zones.rs        → Zonas de memória
│   ├── stats.rs        → Estatísticas
│   └── region.rs       → MemoryRegion
├── vmm/
│   ├── mod.rs
│   ├── mapper.rs       → map_page, unmap_page
│   ├── tlb.rs          → TLB management
│   └── vmm.rs          → AddressSpace
├── alloc/
│   ├── mod.rs
│   ├── bump.rs         → Bump allocator (boot)
│   ├── buddy.rs        → Buddy allocator
│   ├── slab.rs         → Slab allocator
│   └── percpu.rs       → Per-CPU allocator
├── heap/
│   └── mod.rs          → GlobalAlloc
├── cache/
│   ├── mod.rs
│   └── pagecache.rs    → Page cache
├── types/
│   ├── mod.rs
│   ├── vmo.rs          → Virtual Memory Object
│   └── pinned.rs       → Pinned memory
└── ops/
    ├── mod.rs
    └── memops/
        ├── mod.rs
        ├── rust_impl.rs   → Implementação Rust
        └── asm_impl.rs    → Implementação ASM
```

---

## 3. REGRAS

### ❌ NUNCA:
- Usar `f32`/`f64`
- Usar `unwrap()` ou `expect()`
- Acessar memória física sem mapear primeiro
- Assumir identity mapping fora do early boot

### ✅ SEMPRE:
- Usar tipos `PhysAddr` e `VirtAddr` (nunca `u64` raw)
- Verificar alinhamento antes de mapear
- Invalidar TLB após alterar page tables
- Documentar invariantes de segurança

---

## 4. IMPLEMENTAÇÕES

### 4.1 `addr/phys.rs`

```rust
//! Endereço físico type-safe

/// Endereço físico (não pode ser dereferenciado diretamente)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl PhysAddr {
    /// Máscara para endereços físicos válidos (48 bits em x86_64)
    const MASK: u64 = 0x000F_FFFF_FFFF_FFFF;
    
    /// Cria novo endereço físico
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self(addr & Self::MASK)
    }
    
    /// Cria sem validação (use com cuidado)
    #[inline]
    pub const fn new_unchecked(addr: u64) -> Self {
        Self(addr)
    }
    
    /// Retorna valor raw
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
    
    /// Alinha para cima
    #[inline]
    pub const fn align_up(self, align: u64) -> Self {
        Self((self.0 + align - 1) & !(align - 1))
    }
    
    /// Alinha para baixo
    #[inline]
    pub const fn align_down(self, align: u64) -> Self {
        Self(self.0 & !(align - 1))
    }
    
    /// Verifica alinhamento
    #[inline]
    pub const fn is_aligned(self, align: u64) -> bool {
        (self.0 & (align - 1)) == 0
    }
    
    /// Offset
    #[inline]
    pub const fn offset(self, offset: u64) -> Self {
        Self(self.0 + offset)
    }
}

impl core::ops::Add<u64> for PhysAddr {
    type Output = Self;
    fn add(self, rhs: u64) -> Self {
        Self(self.0 + rhs)
    }
}

impl core::ops::Sub<PhysAddr> for PhysAddr {
    type Output = u64;
    fn sub(self, rhs: PhysAddr) -> u64 {
        self.0 - rhs.0
    }
}
```

### 4.2 `addr/virt.rs`

```rust
//! Endereço virtual type-safe

/// Endereço virtual
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    /// Cria novo endereço virtual (canonicaliza)
    #[inline]
    pub const fn new(addr: u64) -> Self {
        // Canonicalização x86_64: bit 47 é extendido para bits 48-63
        let canonical = if (addr >> 47) & 1 == 1 {
            addr | 0xFFFF_0000_0000_0000
        } else {
            addr & 0x0000_FFFF_FFFF_FFFF
        };
        Self(canonical)
    }
    
    /// Cria sem canonicalização
    #[inline]
    pub const fn new_unchecked(addr: u64) -> Self {
        Self(addr)
    }
    
    /// Retorna valor raw
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
    
    /// Converte para ponteiro
    #[inline]
    pub const fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }
    
    /// Converte para ponteiro mutável
    #[inline]
    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
    
    /// Alinha para cima
    #[inline]
    pub const fn align_up(self, align: u64) -> Self {
        Self::new((self.0 + align - 1) & !(align - 1))
    }
    
    /// Alinha para baixo  
    #[inline]
    pub const fn align_down(self, align: u64) -> Self {
        Self::new(self.0 & !(align - 1))
    }
    
    /// Índices de page table
    #[inline]
    pub const fn p4_index(self) -> usize {
        ((self.0 >> 39) & 0x1FF) as usize
    }
    
    #[inline]
    pub const fn p3_index(self) -> usize {
        ((self.0 >> 30) & 0x1FF) as usize
    }
    
    #[inline]
    pub const fn p2_index(self) -> usize {
        ((self.0 >> 21) & 0x1FF) as usize
    }
    
    #[inline]
    pub const fn p1_index(self) -> usize {
        ((self.0 >> 12) & 0x1FF) as usize
    }
    
    /// Offset dentro da página
    #[inline]
    pub const fn page_offset(self) -> u64 {
        self.0 & 0xFFF
    }
}
```

### 4.3 `pmm/frame.rs`

```rust
//! Frame físico (página de 4KB)

use crate::mm::addr::PhysAddr;
use crate::arch::PAGE_SIZE;

/// Um frame físico de 4KB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysFrame {
    base: PhysAddr,
}

impl PhysFrame {
    /// Cria frame a partir de endereço alinhado
    pub fn from_addr(addr: PhysAddr) -> Option<Self> {
        if addr.is_aligned(PAGE_SIZE as u64) {
            Some(Self { base: addr })
        } else {
            None
        }
    }
    
    /// Cria frame contendo o endereço
    pub fn containing(addr: PhysAddr) -> Self {
        Self {
            base: addr.align_down(PAGE_SIZE as u64),
        }
    }
    
    /// Endereço base do frame
    pub const fn base(&self) -> PhysAddr {
        self.base
    }
    
    /// Número do frame
    pub const fn number(&self) -> u64 {
        self.base.as_u64() / PAGE_SIZE as u64
    }
    
    /// Cria a partir de número de frame
    pub const fn from_number(n: u64) -> Self {
        Self {
            base: PhysAddr::new(n * PAGE_SIZE as u64),
        }
    }
}
```

### 4.4 `vmm/mapper.rs`

```rust
//! Mapeamento de páginas

use crate::mm::{PhysAddr, VirtAddr, PhysFrame, MmError};
use crate::arch::PAGE_SIZE;

/// Flags de mapeamento
#[derive(Debug, Clone, Copy)]
pub struct MapFlags {
    bits: u64,
}

impl MapFlags {
    pub const PRESENT: Self = Self { bits: 1 << 0 };
    pub const WRITABLE: Self = Self { bits: 1 << 1 };
    pub const USER: Self = Self { bits: 1 << 2 };
    pub const NO_EXECUTE: Self = Self { bits: 1 << 63 };
    
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }
    
    pub const fn union(self, other: Self) -> Self {
        Self { bits: self.bits | other.bits }
    }
    
    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }
}

/// Mapeia uma página virtual para um frame físico
/// 
/// # Safety
/// 
/// - `virt` deve estar alinhado a PAGE_SIZE
/// - `phys` deve ser um frame válido
/// - Caller deve garantir que não há conflitos
pub unsafe fn map_page(
    virt: VirtAddr,
    phys: PhysFrame,
    flags: MapFlags,
) -> Result<(), MmError> {
    // Verificar alinhamento
    if !virt.as_u64() % PAGE_SIZE as u64 == 0 {
        return Err(MmError::NotAligned);
    }
    
    // TODO: Implementar mapeamento real nas page tables
    // 1. Walk P4 -> P3 -> P2 -> P1
    // 2. Criar tabelas intermediárias se necessário
    // 3. Escrever entrada P1 com endereço físico + flags
    
    // Invalidar TLB para esta página
    crate::mm::vmm::tlb::invlpg(virt);
    
    Ok(())
}

/// Desmapeia uma página
pub unsafe fn unmap_page(virt: VirtAddr) -> Result<PhysFrame, MmError> {
    // TODO: Implementar
    // 1. Walk page tables
    // 2. Limpar entrada
    // 3. Retornar frame que estava mapeado
    
    crate::mm::vmm::tlb::invlpg(virt);
    
    Err(MmError::NotMapped)
}

/// Traduz endereço virtual para físico
pub fn translate_addr(virt: VirtAddr) -> Option<PhysAddr> {
    // TODO: Implementar walking das page tables
    None
}
```

### 4.5 `vmm/tlb.rs`

```rust
//! Gerenciamento de TLB

use crate::mm::VirtAddr;

/// Invalida entrada TLB para endereço específico
#[inline]
pub fn invlpg(addr: VirtAddr) {
    // SAFETY: invlpg é seguro, apenas invalida cache
    unsafe {
        core::arch::asm!(
            "invlpg [{}]",
            in(reg) addr.as_u64(),
            options(nostack)
        );
    }
}

/// Flush completo do TLB (reload CR3)
#[inline]
pub fn flush_all() {
    // SAFETY: Recarregar CR3 é seguro
    unsafe {
        let cr3 = crate::arch::x86_64::cpu::Cpu::read_cr3();
        crate::arch::x86_64::cpu::Cpu::write_cr3(cr3);
    }
}
```

### 4.6 `heap/mod.rs`

```rust
//! Heap do kernel (GlobalAlloc)

use core::alloc::{GlobalAlloc, Layout};
use crate::sync::Spinlock;

/// Alocador global do kernel
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Heap com lock
pub struct LockedHeap {
    inner: Spinlock<HeapInner>,
}

struct HeapInner {
    start: usize,
    end: usize,
    next: usize,
}

impl LockedHeap {
    pub const fn empty() -> Self {
        Self {
            inner: Spinlock::new(HeapInner {
                start: 0,
                end: 0,
                next: 0,
            }),
        }
    }
    
    /// Inicializa o heap
    /// 
    /// # Safety
    /// 
    /// - `start` deve ser endereço válido mapeado
    /// - Região [start, start+size) deve ser usável
    pub unsafe fn init(&self, start: usize, size: usize) {
        let mut inner = self.inner.lock();
        inner.start = start;
        inner.end = start + size;
        inner.next = start;
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut inner = self.inner.lock();
        
        // Alinhar
        let align = layout.align();
        let aligned = (inner.next + align - 1) & !(align - 1);
        let end = aligned + layout.size();
        
        if end > inner.end {
            // Sem memória
            core::ptr::null_mut()
        } else {
            inner.next = end;
            aligned as *mut u8
        }
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator não libera memória
        // TODO: Implementar slab/buddy para liberação real
    }
}

/// Handler de erro de alocação
#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    crate::kerror!("Falha de alocação! size=", layout.size() as u64);
    panic!("out of memory");
}

/// Inicializa o heap
pub fn init() {
    // TODO: Alocar região de memória para heap
    // unsafe { ALLOCATOR.init(heap_start, heap_size); }
    crate::kinfo!("Heap inicializado");
}
```

### 4.7 `error.rs`

```rust
//! Erros de memória

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmError {
    /// Sem memória disponível
    OutOfMemory,
    /// Endereço não alinhado
    NotAligned,
    /// Região já mapeada
    AlreadyMapped,
    /// Região não mapeada
    NotMapped,
    /// Endereço inválido
    InvalidAddress,
    /// Permissão negada
    PermissionDenied,
    /// Frame não disponível
    FrameNotAvailable,
}
```

---

## 5. ORDEM DE IMPLEMENTAÇÃO

1. `addr/phys.rs` e `addr/virt.rs` - Tipos de endereço
2. `config.rs` - Constantes
3. `error.rs` - Erros
4. `pmm/frame.rs` - Frame abstraction
5. `pmm/bitmap.rs` - Tracking de frames
6. `vmm/tlb.rs` - TLB ops
7. `vmm/mapper.rs` - Mapeamento
8. `heap/mod.rs` - GlobalAlloc
9. `alloc/bump.rs` - Boot allocator
10. `alloc/slab.rs` - Object allocator

---

## 6. DEPENDÊNCIAS

Pode importar de:
- `crate::arch` (PAGE_SIZE, Cpu::read_cr3)
- `crate::klib`
- `crate::sync`

NÃO pode importar de:
- `crate::sched`
- `crate::fs`
- `crate::ipc`

---

## 7. CHECKLIST

- [ ] PhysAddr e VirtAddr são newtypes (não `u64` raw)
- [ ] Toda operação de memória física passa por PMM
- [ ] TLB é invalidada após alterar page tables
- [ ] GlobalAlloc está implementado
- [ ] Nenhum `unwrap()`
