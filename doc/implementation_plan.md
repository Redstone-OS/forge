# Plano de Implementação: Subsistema de Memória Moderno para Forge

## 📋 Visão Geral

Este documento descreve a arquitetura e implementação de um subsistema de memória **moderno, robusto e profissional** para o Kernel Forge, seguindo os princípios de design mais avançados da indústria, indo além do modelo clássico do Linux.

---

## 🎯 Princípios Fundamentais

| Princípio | Descrição |
|-----------|-----------|
| **Isolamento Absoluto** | Cada processo nasce com address space vazio. Nada herdado por acidente. |
| **Lazy Everything** | Página só existe quando alguém toca. Page fault é caminho normal. |
| **Ownership Explícito** | Cada frame físico tem dono(s). Refcount sempre. Sem dono → livre. |
| **Separação Clara** | Virtual ≠ físico. Kernel ≠ user. Dado ≠ permissão. |
| **Metadados Ricos** | Cada região tem intenção (heap, stack, framebuffer). Kernel decide baseado nisso. |
| **Zero Compartilhamento Implícito** | Compartilhar memória é sempre explícito via syscall. |

---

## 🏗️ Arquitetura Proposta

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           KERNEL MEMORY SUBSYSTEM                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         KERNEL HEAP (HHDM)                          │    │
│  │  Buddy + Slab → Box, Vec, Arc para estruturas internas do kernel    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    ↑                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    ADDRESS SPACE MANAGER                            │    │
│  │  AddressSpace (CR3) + VMA List + Page Fault Handler                 │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    ↑                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    PAGE FRAME MANAGER (PFM)                         │    │
│  │  Frame → Owner → RefCount → State (Free/Used/COW/Pinned)            │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    ↑                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    PHYSICAL MEMORY MANAGER (PMM)                    │    │
│  │  Bitmap → PhysFrame → Boot-time allocation                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 📐 Layout de Memória Virtual (x86_64)

```
┌────────────────────────────────────────┬──────────────────────────────────┐
│           USERSPACE (0-128TB)          │        KERNEL (128TB+)           │
├────────────────────────────────────────┼──────────────────────────────────┤
│ 0x0000_0000_0000_0000                  │ 0xFFFF_8000_0000_0000            │
│ └── [NÃO MAPEADO - Guard Page]         │ └── HHDM (Direct Map RAM)        │
│                                        │     Toda RAM física mapeada      │
│ 0x0000_0000_0040_0000 (4MB+)           │     phys_to_virt(p) = HHDM + p   │
│ └── ELF Code/Data                      │                                  │
│     (Mapeado sob demanda por VMA)      │ 0xFFFF_9000_0000_0000            │
│                                        │ └── Kernel Heap                  │
│ 0x0000_0001_0000_0000 (4GB+)           │     (Buddy + Slab)               │
│ └── Heap (brk() / mmap anônimo)        │                                  │
│     Base dinâmica (ASLR)               │ 0xFFFF_9100_0000_0000            │
│     Cresce via VMA expandido           │ └── Kernel Stacks (per-task)     │
│                                        │                                  │
│ 0x0000_7000_0000_0000 (112TB+)         │ 0xFFFF_FE00_0000_0000            │
│ └── mmap() region                      │ └── Scratch Page (temp mapping)  │
│     Shared memory, files, etc.         │                                  │
│                                        │ 0xFFFF_FFFF_8000_0000            │
│ 0x0000_7FFF_FFFF_F000                  │ └── Kernel Text/Data (-2GB)      │
│ └── Stack (cresce para baixo)          │     (Link address do ELF)        │
│     ASLR aplicado                      │                                  │
└────────────────────────────────────────┴──────────────────────────────────┘
```

> [!IMPORTANT]
> O HHDM (Higher Half Direct Map) é **obrigatório** para eliminar qualquer dependência de identity map na metade inferior. Isso permite que cada processo tenha userspace completamente isolado.

---

## 🧩 Componentes Principais

### 1. Higher Half Direct Map (HHDM)

**Arquivo:** `forge/src/mm/hhdm.rs`

O HHDM mapeia toda a RAM física em uma região fixa do kernel space.

```rust
/// Base do Higher Half Direct Map
pub const HHDM_BASE: u64 = 0xFFFF_8000_0000_0000;

/// Converte endereço físico para virtual (HHDM)
#[inline(always)]
pub fn phys_to_virt<T>(phys: u64) -> *mut T {
    (HHDM_BASE + phys) as *mut T
}

/// Converte endereço virtual (HHDM) para físico
#[inline(always)]
pub fn virt_to_phys(virt: u64) -> u64 {
    debug_assert!(virt >= HHDM_BASE);
    virt - HHDM_BASE
}
```

**Implementação no Boot:**
- Bootloader (Ignite) mapeia toda RAM detectada em `HHDM_BASE + phys`
- Usa huge pages (2MB) para eficiência
- Global bit setado para não flush no context switch

---

### 2. Page Frame Manager (PFM)

**Arquivo:** `forge/src/mm/pfm/mod.rs`

Substitui o conceito de simples bitmap por um sistema de **ownership explícito**.

```rust
/// Estado de um frame físico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameState {
    /// Frame livre, disponível para alocação
    Free,
    /// Frame usado por exatamente um processo
    Owned { owner: Pid },
    /// Frame compartilhado (COW ou shared memory)
    Shared { ref_count: u32 },
    /// Frame do kernel (não pode ser liberado por userspace)
    Kernel,
    /// Frame pinned (não swappable, não movable)
    Pinned { owner: Pid },
    /// Frame usado por hardware (framebuffer, DMA)
    Device,
}

/// Metadados de um frame físico (Compacto: 32 bytes)
#[repr(C, align(32))]
pub struct FrameInfo {
    /// Estado e Flags compactados (Memory ordering: AcqRel/SeqCst)
    pub state_flags: AtomicU64,
    /// Contador de referências (Atômico)
    pub ref_count: AtomicU32,
    /// Lock fino (TicketLock ou Mutex de 1-byte)
    pub lock: TicketLock,
    /// Reverse Mapping Escalável:
    /// - Se 1-3 refs: Armazena inline (Small Array)
    /// - Se >3 refs: Ponteiro para Hashed bucket / Radix tree compressa
    pub rmap_data: RMapData,
    /// Metadados de NUMA e Zone
    pub numa_node: u16,
    /// Invalidation Counter (Fast-path para TLB shootdown)
    pub inv_count: AtomicU32,
}

/// Gerenciador de frames físicos
pub struct PageFrameManager {
    /// Array de metadados alocado no early-boot.
    /// Estratégia: Paginado se RAM > 512GB para economizar memória útil.
    frames: &'static mut [FrameInfo],
    /// Caches per-CPU (Lockless LIFO)
    cpu_caches: [CpuFrameCache; MAX_CPUS],
    stats: PfmStats,
}

impl PageFrameManager {
    /// Aloca frame com ownership explícito (tenta cache local primeiro)
    pub fn alloc_frame(&self, owner: Pid, flags: FrameFlags) -> Option<PhysAddr> { ... }
    
    /// Libera frame (decrementa refcount, limpa rmap se zero)
    pub fn free_frame(&self, frame: PhysAddr, owner: Pid) -> Result<(), PfmError> { ... }
    
    /// Reverse Map: Adiciona referência de um PTE a este frame
    pub fn rmap_add(&self, frame: PhysAddr, pte_ptr: *mut PageTableEntry) { ... }

    /// Reverse Map: Remove todas as referências (para eviction)
    pub fn rmap_unmap_all(&self, frame: PhysAddr) { ... }
}
```

> [!NOTE]
> O PFM é construído **sobre** o PMM existente. O PMM continua gerenciando o bitmap de alocação, mas o PFM adiciona a camada de ownership e refcount.

---

### 3. Address Space Manager

**Arquivo:** `forge/src/mm/aspace/mod.rs`

Cada processo tem seu próprio `AddressSpace`, que gerencia a PML4 e a lista de VMAs.

```rust
/// Address Space de um processo
pub struct AddressSpace {
    /// Endereço físico da PML4 (CR3)
    pml4: PhysAddr,
    /// Lista de VMAs ordenadas por endereço
    vmas: RBTree<VirtAddr, VMA>,
    /// PID do processo dono
    owner: Pid,
    /// Estatísticas
    stats: AddressSpaceStats,
    /// Lock para a árvore de VMAs (Read-Heavy)
    vma_lock: SpinRwLock<()>,
    /// Lock para as tabelas de páginas (Escrita/Manutenção)
    table_lock: SpinLock<()>,
    /// PCID (Process Context ID) atribuído a este ASpace
    pcid: u16,
    /// Generation counter para TLB batching
    tlb_gen: AtomicU64,
}

impl AddressSpace {
    /// Cria novo address space VAZIO para userspace
    /// Kernel half é sempre copiado, userspace é completamente vazio
    pub fn new(owner: Pid) -> Result<Self, MmError> {
        let pml4 = PageFrameManager::alloc_frame(Pid::KERNEL, FrameFlags::KERNEL)?;
        unsafe {
            // Zerar toda a PML4
            memzero(phys_to_virt(pml4), PAGE_SIZE);
            // Copiar APENAS kernel half (entries 256-511)
            copy_kernel_mappings(pml4);
        }
        Ok(Self {
            pml4,
            vmas: RBTree::new(),
            owner,
            stats: AddressSpaceStats::default(),
            lock: SpinRwLock::new(()),
        })
    }
    
    /// Mapeia nova região (cria VMA)
    pub fn map_region(
        &mut self,
        hint: Option<VirtAddr>,
        size: usize,
        prot: Protection,
        flags: VmaFlags,
        intent: MemoryIntent,
    ) -> Result<VirtAddr, MmError> { ... }
    
    /// Remove mapeamento
    pub fn unmap_region(&mut self, addr: VirtAddr, size: usize) -> Result<(), MmError> { ... }
    
    /// Trata page fault
    pub fn handle_fault(
        &mut self, 
        addr: VirtAddr, 
        access: AccessType
    ) -> Result<PhysAddr, FaultResult> { ... }
}
```

---

### 4. Virtual Memory Area (VMA)

**Arquivo:** `forge/src/mm/aspace/vma.rs`

Cada região de memória virtual é descrita por uma VMA com **intenção semântica**.

```rust
/// Intenção de uso da memória
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryIntent {
    /// Código executável (ELF .text)
    Code,
    /// Dados inicializados (ELF .data)
    Data,
    /// Dados não inicializados (ELF .bss)
    Bss,
    /// Heap do processo
    Heap,
    /// Stack do processo
    Stack,
    /// Arquivo mapeado (read-only shared)
    FileReadOnly,
    /// Arquivo mapeado (private COW)
    FilePrivate,
    /// Shared memory (IPC)
    SharedMemory,
    /// Framebuffer / DMA buffer
    DeviceBuffer,
    /// Guard page (não mapeável, causa SIGSEGV)
    Guard,
}

/// Virtual Memory Area
pub struct VMA {
    /// Endereço virtual inicial (page-aligned)
    pub start: VirtAddr,
    /// Endereço virtual final (exclusive, page-aligned)  
    pub end: VirtAddr,
    /// Proteção (Read/Write/Execute)
    pub protection: Protection,
    /// Flags de comportamento
    pub flags: VmaFlags,
    /// Intenção semântica
    pub intent: MemoryIntent,
    /// Backing (anônimo, arquivo, ou VMO)
    pub backing: VmaBacking,
    /// Estatísticas
    pub stats: VmaStats,
}

/// Backing de uma VMA
pub enum VmaBacking {
    /// Memória anônima (zero-fill on demand)
    Anonymous,
    /// Arquivo mapeado
    File { 
        vnode: Arc<VNode>,
        offset: u64,
    },
    /// VMO (Virtual Memory Object)
    Vmo { 
        vmo: Arc<VMO>,
        offset: usize,
    },
}

/// Flags de VMA
bitflags! {
    pub struct VmaFlags: u32 {
        /// Região pode crescer (heap)
        const GROWABLE = 1 << 0;
        /// Região cresce para baixo (stack)
        const GROWS_DOWN = 1 << 1;
        /// Copy-on-write
        const COW = 1 << 2;
        /// Shared entre processos
        const SHARED = 1 << 3;
        /// Locked in memory (não swappable)
        const LOCKED = 1 << 4;
        /// Hint: streaming access
        const STREAMING = 1 << 5;
        /// Hint: descartável se pressão
        const DISCARDABLE = 1 << 6;
        /// Nunca fazer COW (sempre private)
        const NO_COW = 1 << 7;
    }
}
```

---

### 5. Page Fault Handler

**Arquivo:** `forge/src/mm/fault.rs`

O page fault é o **mecanismo central** para alocação lazy e COW.

```rust
/// Resultado de um page fault
pub enum FaultResult {
    /// Fault resolvido, continuar execução
    Resolved(PhysAddr),
    /// Fault devido a COW, página copiada
    CowResolved(PhysAddr),
    /// Região não mapeada, matar processo
    SegmentationFault,
    /// Violação de proteção, matar processo
    ProtectionViolation,
    /// Stack overflow, tentar expandir ou matar
    StackOverflow,
    /// OOM, matar processo ou aguardar
    OutOfMemory,
}

/// Handler principal de page fault
pub fn handle_page_fault(
    addr: VirtAddr,
    error_code: PageFaultError,
    task: &mut Task,
) -> FaultResult {
    let aspace = &mut task.address_space;
    
    // 1. Encontrar VMA que contém o endereço
    let vma = match aspace.find_vma(addr) {
        Some(v) => v,
        None => {
            // Verificar se é stack que pode crescer
            if aspace.can_expand_stack(addr) {
                aspace.expand_stack(addr)?;
                return handle_page_fault(addr, error_code, task);
            }
            return FaultResult::SegmentationFault;
        }
    };
    
    // 2. Verificar proteção
    if !vma.protection.permits(error_code.access_type) {
        return FaultResult::ProtectionViolation;
    }
    
    // 3. Resolver baseado no estado da página
    match vma.get_page_state(addr) {
        PageState::NotPresent => {
            // Alocação lazy
            let frame = allocate_and_map(aspace, vma, addr)?;
            FaultResult::Resolved(frame)
        }
        PageState::CopyOnWrite if error_code.is_write => {
            // COW: copiar página
            let new_frame = copy_on_write(aspace, vma, addr)?;
            FaultResult::CowResolved(new_frame)
        }
        _ => FaultResult::ProtectionViolation,
    }
}
```

---

### 6. Syscalls de Memória

**Arquivo:** [forge/src/syscall/memory/mod.rs](file:///D:/Github/RedstoneOS/forge/src/syscall/memory/mod.rs)

API userspace para gerenciar memória.

```rust
// ============================================================================
// MMAP - Mapear região de memória
// ============================================================================

/// sys_mmap(addr, size, prot, flags, fd, offset) -> Result<VirtAddr>
pub fn sys_mmap(
    hint: usize,
    size: usize,
    prot: u32,
    flags: u32,
    fd: i32,
    offset: u64,
) -> SysResult<usize> {
    let task = current_task();
    let mut aspace = task.address_space.lock();
    
    let protection = Protection::from_bits(prot)?;
    let vma_flags = VmaFlags::from_syscall(flags)?;
    
    // Determinar backing
    let backing = if fd >= 0 {
        let file = task.get_file(fd)?;
        VmaBacking::File { vnode: file.vnode.clone(), offset }
    } else {
        VmaBacking::Anonymous
    };
    
    // Determinar intent baseado em flags
    let intent = infer_intent(protection, vma_flags, backing);
    
    let addr = aspace.map_region(
        if hint == 0 { None } else { Some(VirtAddr::new(hint as u64)) },
        size,
        protection,
        vma_flags,
        intent,
    )?;
    
    Ok(addr.as_u64() as usize)
}

// ============================================================================
// MPROTECT - Alterar proteção
// ============================================================================

/// sys_mprotect(addr, size, prot) -> Result<()>
pub fn sys_mprotect(addr: usize, size: usize, prot: u32) -> SysResult<usize> { ... }

// ============================================================================
// MUNMAP - Remover mapeamento
// ============================================================================

/// sys_munmap(addr, size) -> Result<()>
pub fn sys_munmap(addr: usize, size: usize) -> SysResult<usize> { ... }

// ============================================================================
// MADVISE - Dicas de uso (Userspace-guided)
// ============================================================================

/// sys_madvise(addr, size, advice) -> Result<()>
pub fn sys_madvise(addr: usize, size: usize, advice: i32) -> SysResult<usize> { ... }

---

## 🚀 Melhorias Industrial-Grade (O que nos separa de um kernel "toy")

### 1. Reverse Mappings (RMAP)
Para cada frame físico, o kernel mantém uma lista de todos os PTEs que apontam para ele. 
- **Utilidade:** Quando precisamos liberar um frame (evict) ou trocá-lo (swap), o kernel sabe exatamente quais processos atualizar, sem varrer todas as tabelas de páginas.
- **Implementação:** Lista encadeada de `(ASpace*, VirtAddr)` no `FrameInfo`.

### 2. SMP & TLB Shootdown (Batching + PCID)
Em sistemas multicore, a invalidação de TLB é o maior gargalo de sincronização.
- **Batching:** Não enviamos um IPI para cada página removida. Acumulamos as invalidações no `AddressSpace` e enviamos um único IPI "flush range" ao final da operação (ex: `munmap` de 1GB).
- **PCID (Process Context Identifiers):** Usamos tags de hardware no TLB para evitar flush total no context switch e permitir invalidações seletivas.
- **Fast-path:** Invalidation counter lockless para pular IPIs se o ASpace não estiver ativo em outros cores.

### 3. Page Reclaim (LRU / CLOCK-Pro)
- **Aging:** Implementar CLOCK-Pro ou Two-list LRU (Active/Inactive) para distinguir páginas "quentes" de "frias".
- **kswapd:** Thread dedicada que acorda quando a RAM atinge o "low watermark" e dorme no "high watermark".
- **OOM Killer:** Heurística baseada no custo de recreação da task vs benefício de RAM liberada.

### 4. Segurança & Atômicos (Memory Barriers)
- **Memory Ordering:** Todas as transições de `FrameInfo` (Free -> Owned) devem usar `Release` ordering, e leituras no fault handler `Acquire` ordering para garantir visibilidade em SMP.
- **Zero-on-Alloc:** Garantir que o buffer seja zerado usando instruções NT (non-temporal) se possível para não poluir o cache.

### 5. NUMA-Awareness
Em servidores modernos, a RAM não é uniforme.
- **Policy:** O alocador tenta entregar RAM fisicamente próxima ao core que a solicitou (Node Locality).

### 6. File-backed VMAs & Page Cache
As páginas mapeadas de arquivos precisam estar em sincronia com o Page Cache do kernel.
- **Integração:** O `rmap` permite que o kernel encontre todos os processos que mapearam um arquivo para fazer o flush/writeback quando o Page Cache decide gravar no disco.
- **Mecanismo:** `VmaBacking::File` aponta para o objeto de cache do VNode.

### 7. IOMMU & DMA Integration
Drivers de hardware (GPU, NIC) precisam de memória contígua e visível via IOMMU.
- **Pinned Frames:** Frames de DMA são marcados como `Pinned` no `FrameInfo` e ignorados pelo `kswapd`.
- **Coerência:** O `rmap` deve rastrear se um frame está mapeado em um IOMMU group para invalidar caches de hardware se a página for movida.

### 8. Huge Pages (2MB / 1GB)
- **Split/Merge:** O kernel deve ser capaz de dividir uma Huge Page em 4KB pages se um processo chamar `mprotect` em apenas uma parte dela.
- **Alignment:** O alocador PFM deve garantir alinhamento natural (2MB align) para Huge Pages sem fragmentação excessiva.

---

## 🔒 Hierarquia de Locks e Invariantes

Para evitar deadlocks (especialmente em rmap vs page fault), a seguinte ordem **estrita** deve ser seguida:

1. **AddressSpace Lock** (vma_lock)
2. **VMA Lock** (se aplicável)
3. **AddressSpace Table Lock** (table_lock)
4. **FrameInfo Lock** (per-frame)

**Invariante:** Nunca tente adquirir um lock de `AddressSpace` segurando um lock de `FrameInfo` sem usar `try_lock`. Se o reclaim precisar travar um ASpace que já está travado, ele deve recuar (backoff).

---

---

## 📊 Fases de Implementação

### Fase 1: Fundação Industrial (HHDM + Early Allocator) 
**Estimativa: 4-6 dias**

| Item | Descrição | Arquivos |
|------|-----------|----------|
| 1.1 | Atualizar Bootloader para criar HHDM | `ignite/` |
| 1.2 | Early Boot Allocator (alocar FrameInfo array compacto) | `forge/src/mm/early.rs` |
| 1.3 | Implementar HHDM (Direct Map) com 1GB pages | `forge/src/mm/hhdm.rs` |
| 1.4 | Suporte a Huge Pages (2MB/1GB) no PFM | `forge/src/mm/vmm/huge.rs` |
| 1.5 | SMP: IPI Batching Engine para TLB | `forge/src/arch/x86_64/smp/tlb.rs` |
| 1.6 | PCID Management (x86_64) | `forge/src/arch/x86_64/vmm/pcid.rs` |

**Checkpoint:** Kernel boota com HHDM, suporte a Huge Pages e infra de TLB batching pronta.

---

### Fase 2: PFM com RMap Escalável
**Estimativa: 4-6 dias**

| Item | Descrição | Arquivos |
|------|-----------|----------|
| 2.1 | `FrameInfo` (32 bytes) com Atomic State | `forge/src/mm/pfm/frame.rs` |
| 2.2 | RMap: Small Array + Hashed Overflow | `forge/src/mm/pfm/rmap.rs` |
| 2.3 | Caches Per-CPU Lockless | `forge/src/mm/pfm/cache.rs` |
| 2.4 | IOMMU API & Pinned coordination | `forge/src/mm/pfm/iommu.rs` |
| 2.5 | Zero-on-Alloc (Background thread opcional) | `forge/src/mm/pfm/zero.rs` |

**Checkpoint:** Alocação de frames escalável e metadados preparados para IOMMU e DMA.

---

### Fase 3: Address Space & Lock Strategy
**Estimativa: 5-7 dias**

| Item | Descrição | Arquivos |
|------|-----------|----------|
| 3.1 | Implementar hierarquia de locks (VMA vs Table) | `forge/src/mm/aspace/mod.rs` |
| 3.2 | RBTree balanceada para VMAs | `forge/src/mm/aspace/rbtree.rs` |
| 3.3 | Integração de TLB Shootdown no unmap | `forge/src/mm/vmm/tlb.rs` |
| 3.4 | SMAP/SMEP Enforcing | `forge/src/arch/x86_64/cpu.rs` |
| 3.5 | Criar estrutura `VMA` | `forge/src/mm/aspace/vma.rs` |
| 3.6 | Implementar `AddressSpace` | `forge/src/mm/aspace/mod.rs` |
| 3.7 | Integrar com Task | [forge/src/sched/task/entity.rs](file:///D:/Github/RedstoneOS/forge/src/sched/task/entity.rs) |
| 3.8 | Novo [spawn()](file:///D:/Github/RedstoneOS/forge/src/sched/exec/loader.rs#36-266) usando AddressSpace | [forge/src/sched/exec/loader.rs](file:///D:/Github/RedstoneOS/forge/src/sched/exec/loader.rs) |
| 3.9 | Testes: spawn processos isolados | - |

**Checkpoint:** Processos isolados com proteção de kernel e sincronização SMP.

---

### Fase 4: Page Fault Handler
**Estimativa: 3-4 dias**

| Item | Descrição | Arquivos |
|------|-----------|----------|
| 4.1 | Refatorar handler de #PF | [forge/src/arch/x86_64/interrupts.rs](file:///D:/Github/RedstoneOS/forge/src/arch/x86_64/interrupts.rs) |
| 4.2 | Implementar lazy allocation | `forge/src/mm/fault.rs` |
| 4.3 | Implementar COW | `forge/src/mm/fault.rs` |
| 4.4 | Stack expansion | `forge/src/mm/fault.rs` |
| 4.5 | Testes: lazy alloc, COW | - |

**Checkpoint:** Páginas alocadas sob demanda, COW funcional.

---

### Fase 5: Syscalls de Memória
**Estimativa: 2-3 dias**

| Item | Descrição | Arquivos |
|------|-----------|----------|
| 5.1 | Implementar `sys_mmap` | `forge/src/syscall/memory/mmap.rs` |
| 5.2 | Implementar `sys_munmap` | `forge/src/syscall/memory/mmap.rs` |
| 5.3 | Implementar [sys_mprotect](file:///D:/Github/RedstoneOS/forge/src/syscall/memory/alloc.rs#163-169) | `forge/src/syscall/memory/mmap.rs` |
| 5.4 | Implementar `sys_madvise` | `forge/src/syscall/memory/madvise.rs` |
| 5.5 | Atualizar SDK (redpowder) | `redpowder/src/mem/` |
| 5.6 | Testes: mmap/munmap userspace | - |

**Checkpoint:** Userspace pode alocar memória via mmap.

---

### Fase 6: Heap Userspace (brk)
**Estimativa: 1-2 dias**

| Item | Descrição | Arquivos |
|------|-----------|----------|
| 6.1 | Implementar `sys_brk` | `forge/src/syscall/memory/brk.rs` |
| 6.2 | VMA de heap por processo | `forge/src/mm/aspace/heap.rs` |
| 6.3 | Atualizar SDK allocator | `redpowder/src/mem/heap.rs` |
| 6.4 | Testes: Vec/String em userspace | - |

**Checkpoint:** Heap userspace funciona corretamente isolado.

---

### Fase 7: Shared Memory
**Estimativa: 2-3 dias**

| Item | Descrição | Arquivos |
|------|-----------|----------|
| 7.1 | Criar VMO (Virtual Memory Object) | Já existe: `forge/src/mm/types/vmo.rs` |
| 7.2 | Syscalls de VMO | `forge/src/syscall/memory/vmo.rs` |
| 7.3 | Mapeamento compartilhado | `forge/src/mm/aspace/shared.rs` |
| 7.4 | Testes: IPC via shared memory | - |

**Checkpoint:** Processos podem compartilhar memória explicitamente.

---
### Fase 8: Reclaim & OOM Policy
**Estimativa: 4-6 dias**

| Item | Descrição | Arquivos |
|------|-----------|----------|
| 8.1 | Page Aging (CLOCK-Pro ou 2-List LRU) | `forge/src/mm/reclaim/aging.rs` |
| 8.2 | Eviction Engine (rmap-based unmap) | `forge/src/mm/reclaim/evict.rs` |
| 8.3 | Thread `kswapd`: Pressure handling | `forge/src/mm/reclaim/kswapd.rs` |
| 8.4 | OOM Killer (Heurística: CPU time vs RAM) | `forge/src/mm/reclaim/oom.rs` |
| 8.5 | Swap: Backing store implementation | `forge/src/mm/swap/mod.rs` |

**Checkpoint:** Sistema resiste a pressão de memória com swap e eviction funcional.

---

### Fase 9: Observabilidade & Estresse
**Estimativa: 3-4 dias**

| Item | Descrição | Arquivos |
|------|-----------|----------|
| 9.1 | Tracepoints para Alloc/Fault/Reclaim | `forge/src/mm/trace.rs` |
| 9.2 | KASAN & Fault Injection | `forge/src/mm/debug/` |
| 9.3 | Counters (Shared memory pages, dirty pages) | `forge/src/mm/stats.rs` |
| 9.4 | MMStress e Validação de carga real | `apps/mmstress/` |

---

## 📁 Estrutura de Arquivos Proposta

```
forge/src/mm/
├── mod.rs                    # Re-exports e init()
├── config.rs                 # Constantes (atualizar)
├── error.rs                  # Tipos de erro (atualizar)
│
├── hhdm.rs                   # [NOVO] Higher Half Direct Map
│
├── pfm/                      # [NOVO] Page Frame Manager
│   ├── mod.rs                # API principal
│   ├── frame.rs              # FrameInfo, FrameState
│   ├── alloc.rs              # Alocação com ownership
│   └── refcount.rs           # Gerenciamento de refcount
│
├── aspace/                   # [NOVO] Address Space
│   ├── mod.rs                # AddressSpace
│   ├── vma.rs                # VMA, MemoryIntent
│   ├── rbtree.rs             # RBTree para VMAs
│   ├── heap.rs               # Heap region management
│   └── shared.rs             # Shared memory
│
├── fault.rs                  # [NOVO] Page fault handler
│
├── pmm/                      # Physical Memory Manager (manter)
│   ├── mod.rs
│   ├── bitmap.rs             # Manter como base
│   └── ...
│
├── vmm/                      # Virtual Memory Manager (refatorar)
│   ├── mod.rs
│   ├── mapper.rs             # Atualizar create_new_p4
│   └── ...
│
├── heap/                     # Kernel Heap (manter)
│   └── mod.rs
│
├── types/                    # Tipos (manter/expandir)
│   ├── vmo.rs                # VMO (já existe)
│   └── ...
│
└── alloc/                    # Allocators (manter)
    ├── buddy.rs
    ├── slab.rs
    └── ...
```

---

## ⚠️ Riscos e Mitigações

| Risco | Probabilidade | Impacto | Mitigação |
|-------|---------------|---------|-----------|
| Bootloader não suporta HHDM | Média | Alta | Modificar Ignite antes de começar |
| Deadlocks em rmap/locking circular | Alta | Crítica | Hierarquia estrita de locks (PFM -> ASpace -> VMA) |
| TLB Stale Mappings (SMP) | Média | Crítica | IPIs síncronas para shootdown e barreiras de memória |
| Overhead de metadados (FrameInfo) | Baixa | Média | Alocação no early boot e uso de campos compactos |
| Corrupção por DMA/IOMMU | Média | Alta | Manter frames de hardware Pinned e usar IOMMU API |

---

## ✅ Critérios de Sucesso

1. **Boot completo** com HHDM e userspace vazio
2. **Supervisor + Firefly + Shell** funcionando sem corrupção
3. **Page fault** resolvendo alocações lazy corretamente
4. **COW** funcionando para fork() (quando implementado)
5. **Zero compartilhamento acidental** entre processos
6. **Performance** similar ou melhor que implementação atual
7. **Código limpo** sem gambiarras ou "TODO: fix later"

---

## 📚 Referências Técnicas

- [Intel SDM Volume 3, Chapter 4 - Paging](https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html)
- [Linux mm/ subsystem](https://github.com/torvalds/linux/tree/master/mm)
- [Fuchsia Zircon VMO](https://fuchsia.dev/fuchsia-src/reference/kernel_objects/vm_object)
- [seL4 Memory Management](https://docs.sel4.systems/projects/sel4/api-doc.html)

---

## 🎯 Próximos Passos

1. **Revisar** este plano e aprovar arquitetura
2. **Modificar Ignite** (bootloader) para criar HHDM
3. **Implementar Fase 1** (HHDM no kernel)
4. **Testar** boot básico
5. **Continuar** com fases subsequentes

> [!CAUTION]
> Este é um refactoring **significativo** do subsistema mais crítico do kernel. Cada fase deve ser testada exaustivamente antes de prosseguir. Não há atalhos.
