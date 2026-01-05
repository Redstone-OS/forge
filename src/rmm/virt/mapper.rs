//! # Page Table Mapper
//!
//! Manipulação de page tables x86_64 (PML4 → PDPT → PD → PT).
//!
//! ## Estrutura de Page Table (x86_64)
//!
//! Cada nível tem 512 entradas de 8 bytes = 4KB por tabela.
//!
//! ```text
//! PML4[512] ──┬──► PDPT[512] ──┬──► PD[512] ──┬──► PT[512] ──► Página 4KB
//!             │                │              │
//!             │                │              └──► Huge Page 2MB
//!             │                │
//!             │                └──► Giant Page 1GB
//!             │
//!             └──► (Raramente usado, 512GB por entry)
//! ```
//!
//! ## Page Table Entry Format
//!
//! ```text
//! Bit 63      : No Execute (NX)
//! Bits 62-52  : Available
//! Bits 51-12  : Physical Address (aligned to 4KB)
//! Bit 11-9    : Available
//! Bit 8       : Global
//! Bit 7       : Page Size (Huge)
//! Bit 6       : Dirty
//! Bit 5       : Accessed
//! Bit 4       : Cache Disable
//! Bit 3       : Write Through
//! Bit 2       : User
//! Bit 1       : Writable
//! Bit 0       : Present
//! ```

use crate::rmm::addr::{PhysAddr, VirtAddr};
use crate::rmm::config::*;
use crate::rmm::error::{RmmError, RmmResult};
use crate::rmm::phys::{self, AllocFlags, FrameOwner};
use crate::rmm::zone::Zone;

use super::hhdm;
use super::tlb;

// =============================================================================
// MapFlags
// =============================================================================

/// Flags de mapeamento de página
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct MapFlags(u64);

impl MapFlags {
    /// Nenhuma flag
    pub const NONE: Self = Self(0);
    /// Página presente em memória
    pub const PRESENT: Self = Self(PTE_PRESENT);
    /// Página escrevível
    pub const WRITABLE: Self = Self(PTE_WRITABLE);
    /// Página acessível por userspace
    pub const USER: Self = Self(PTE_USER);
    /// Write-through caching
    pub const WRITE_THROUGH: Self = Self(PTE_WRITE_THROUGH);
    /// Cache desabilitado
    pub const NO_CACHE: Self = Self(PTE_NO_CACHE);
    /// Huge page (2MB ou 1GB)
    pub const HUGE: Self = Self(PTE_HUGE);
    /// Página global (não flush no CR3 switch)
    pub const GLOBAL: Self = Self(PTE_GLOBAL);
    /// No-Execute
    pub const NO_EXEC: Self = Self(PTE_NO_EXEC);

    // Combinações comuns
    /// Kernel read-only
    pub const KERNEL_RO: Self = Self(PTE_PRESENT | PTE_GLOBAL);
    /// Kernel read-write
    pub const KERNEL_RW: Self = Self(PTE_PRESENT | PTE_WRITABLE | PTE_GLOBAL);
    /// Kernel executable
    pub const KERNEL_RX: Self = Self(PTE_PRESENT | PTE_GLOBAL);
    /// User read-only
    pub const USER_RO: Self = Self(PTE_PRESENT | PTE_USER);
    /// User read-write
    pub const USER_RW: Self = Self(PTE_PRESENT | PTE_WRITABLE | PTE_USER);
    /// User executable
    pub const USER_RX: Self = Self(PTE_PRESENT | PTE_USER);
    /// User read-write-execute
    pub const USER_RWX: Self = Self(PTE_PRESENT | PTE_WRITABLE | PTE_USER);
    /// Device memory (no cache)
    pub const DEVICE: Self = Self(PTE_PRESENT | PTE_WRITABLE | PTE_NO_CACHE | PTE_GLOBAL);

    #[inline]
    pub const fn bits(&self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub const fn remove(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Verifica se é uma página presente
    #[inline]
    pub const fn is_present(&self) -> bool {
        (self.0 & PTE_PRESENT) != 0
    }

    /// Verifica se é página de usuário
    #[inline]
    pub const fn is_user(&self) -> bool {
        (self.0 & PTE_USER) != 0
    }

    /// Verifica se é escrevível
    #[inline]
    pub const fn is_writable(&self) -> bool {
        (self.0 & PTE_WRITABLE) != 0
    }

    /// Verifica se é executável
    #[inline]
    pub const fn is_executable(&self) -> bool {
        (self.0 & PTE_NO_EXEC) == 0
    }
}

// =============================================================================
// Page Table Entry
// =============================================================================

/// Entrada de page table (8 bytes)
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Entry vazia (não presente)
    pub const EMPTY: Self = Self(0);

    /// Cria entry com endereço físico e flags
    #[inline]
    pub fn new(phys: PhysAddr, flags: MapFlags) -> Self {
        Self((phys.as_u64() & PTE_ADDR_MASK) | flags.bits())
    }

    /// Retorna endereço físico da entrada
    #[inline]
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & PTE_ADDR_MASK)
    }

    /// Retorna flags da entrada
    #[inline]
    pub fn flags(&self) -> MapFlags {
        MapFlags(self.0 & !PTE_ADDR_MASK)
    }

    /// Verifica se entrada está presente
    #[inline]
    pub fn is_present(&self) -> bool {
        (self.0 & PTE_PRESENT) != 0
    }

    /// Verifica se é huge page
    #[inline]
    pub fn is_huge(&self) -> bool {
        (self.0 & PTE_HUGE) != 0
    }

    /// Verifica se está vazia
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Define flags
    #[inline]
    pub fn set_flags(&mut self, flags: MapFlags) {
        self.0 = (self.0 & PTE_ADDR_MASK) | flags.bits();
    }

    /// Adiciona flags
    #[inline]
    pub fn add_flags(&mut self, flags: MapFlags) {
        self.0 |= flags.bits();
    }

    /// Remove flags
    #[inline]
    pub fn remove_flags(&mut self, flags: MapFlags) {
        self.0 &= !flags.bits();
    }

    /// Valor raw
    #[inline]
    pub fn raw(&self) -> u64 {
        self.0
    }
}

// =============================================================================
// Page Table
// =============================================================================

/// Page table com 512 entradas
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    /// Cria page table vazia
    pub const fn empty() -> Self {
        Self {
            entries: [PageTableEntry::EMPTY; 512],
        }
    }

    /// Acessa entrada pelo índice
    #[inline]
    pub fn entry(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }

    /// Acessa entrada mutável pelo índice
    #[inline]
    pub fn entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }

    /// Itera sobre entradas presentes
    pub fn iter_present(&self) -> impl Iterator<Item = (usize, &PageTableEntry)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_present())
    }

    /// Conta entradas presentes
    pub fn count_present(&self) -> usize {
        self.entries.iter().filter(|e| e.is_present()).count()
    }

    /// Zera todas as entradas
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = PageTableEntry::EMPTY;
        }
    }
}

// =============================================================================
// Índices de Page Table
// =============================================================================

/// Extrai índice PML4 do endereço virtual (bits 39-47)
#[inline]
pub fn pml4_index(virt: VirtAddr) -> usize {
    ((virt.as_u64() >> 39) & 0x1FF) as usize
}

/// Extrai índice PDPT do endereço virtual (bits 30-38)
#[inline]
pub fn pdpt_index(virt: VirtAddr) -> usize {
    ((virt.as_u64() >> 30) & 0x1FF) as usize
}

/// Extrai índice PD do endereço virtual (bits 21-29)
#[inline]
pub fn pd_index(virt: VirtAddr) -> usize {
    ((virt.as_u64() >> 21) & 0x1FF) as usize
}

/// Extrai índice PT do endereço virtual (bits 12-20)
#[inline]
pub fn pt_index(virt: VirtAddr) -> usize {
    ((virt.as_u64() >> 12) & 0x1FF) as usize
}

/// Extrai offset dentro da página (bits 0-11)
#[inline]
pub fn page_offset(virt: VirtAddr) -> usize {
    (virt.as_u64() & 0xFFF) as usize
}

// =============================================================================
// CR3 (Page Table Root)
// =============================================================================

/// Lê CR3 (endereço físico da PML4)
#[inline]
pub fn read_cr3() -> PhysAddr {
    let cr3: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    }
    PhysAddr::new(cr3 & PTE_ADDR_MASK)
}

/// Escreve CR3 (troca page table root)
///
/// # Safety
///
/// A nova PML4 deve ser válida e mapear pelo menos o código atual.
#[inline]
pub unsafe fn write_cr3(pml4: PhysAddr) {
    core::arch::asm!("mov cr3, {}", in(reg) pml4.as_u64(), options(nostack));
}

// =============================================================================
// Operações de Mapeamento
// =============================================================================

/// Mapeia uma página virtual para física
///
/// Cria tabelas intermediárias conforme necessário.
pub fn map_page(virt: VirtAddr, phys: PhysAddr, flags: MapFlags) -> RmmResult<()> {
    if !virt.is_page_aligned() || !phys.is_page_aligned() {
        return Err(RmmError::NotAligned);
    }

    let pml4_phys = read_cr3();
    let pml4 = unsafe { &mut *(hhdm::phys_to_virt(pml4_phys.as_u64()) as *mut PageTable) };

    // Obter ou criar PDPT
    let pdpt = get_or_create_table(pml4, pml4_index(virt), flags)?;

    // Obter ou criar PD
    let pd = get_or_create_table(pdpt, pdpt_index(virt), flags)?;

    // Obter ou criar PT
    let pt = get_or_create_table(pd, pd_index(virt), flags)?;

    // Mapear página
    let entry = pt.entry_mut(pt_index(virt));
    if entry.is_present() {
        return Err(RmmError::AlreadyMapped);
    }

    *entry = PageTableEntry::new(phys, flags.union(MapFlags::PRESENT));

    // Flush TLB para esta página
    tlb::flush_tlb(virt);

    Ok(())
}

/// Remove mapeamento de página
///
/// Retorna o endereço físico que estava mapeado.
pub fn unmap_page(virt: VirtAddr) -> RmmResult<Option<PhysAddr>> {
    if !virt.is_page_aligned() {
        return Err(RmmError::NotAligned);
    }

    let pml4_phys = read_cr3();
    let pml4 = unsafe { &mut *(hhdm::phys_to_virt(pml4_phys.as_u64()) as *mut PageTable) };

    // Walk page tables
    let pml4_entry = pml4.entry(pml4_index(virt));
    if !pml4_entry.is_present() {
        return Ok(None);
    }

    let pdpt = unsafe { &mut *(hhdm::phys_to_virt(pml4_entry.addr().as_u64()) as *mut PageTable) };
    let pdpt_entry = pdpt.entry(pdpt_index(virt));
    if !pdpt_entry.is_present() {
        return Ok(None);
    }

    let pd = unsafe { &mut *(hhdm::phys_to_virt(pdpt_entry.addr().as_u64()) as *mut PageTable) };
    let pd_entry = pd.entry(pd_index(virt));
    if !pd_entry.is_present() {
        return Ok(None);
    }

    let pt = unsafe { &mut *(hhdm::phys_to_virt(pd_entry.addr().as_u64()) as *mut PageTable) };
    let pt_entry = pt.entry_mut(pt_index(virt));

    if !pt_entry.is_present() {
        return Ok(None);
    }

    let phys = pt_entry.addr();
    *pt_entry = PageTableEntry::EMPTY;

    // Flush TLB
    tlb::flush_tlb(virt);

    Ok(Some(phys))
}

/// Traduz endereço virtual para físico
///
/// Faz page walk e retorna o endereço físico correspondente.
pub fn translate(virt: VirtAddr) -> Option<PhysAddr> {
    let pml4_phys = read_cr3();
    let pml4 = unsafe { &*(hhdm::phys_to_virt(pml4_phys.as_u64()) as *const PageTable) };

    // Level 4: PML4
    let pml4_entry = pml4.entry(pml4_index(virt));
    if !pml4_entry.is_present() {
        return None;
    }

    // Level 3: PDPT
    let pdpt = unsafe { &*(hhdm::phys_to_virt(pml4_entry.addr().as_u64()) as *const PageTable) };
    let pdpt_entry = pdpt.entry(pdpt_index(virt));
    if !pdpt_entry.is_present() {
        return None;
    }
    if pdpt_entry.is_huge() {
        // 1GB page
        let base = pdpt_entry.addr().as_u64() & !(GIANT_PAGE_SIZE as u64 - 1);
        let offset = virt.as_u64() & (GIANT_PAGE_SIZE as u64 - 1);
        return Some(PhysAddr::new(base + offset));
    }

    // Level 2: PD
    let pd = unsafe { &*(hhdm::phys_to_virt(pdpt_entry.addr().as_u64()) as *const PageTable) };
    let pd_entry = pd.entry(pd_index(virt));
    if !pd_entry.is_present() {
        return None;
    }
    if pd_entry.is_huge() {
        // 2MB page
        let base = pd_entry.addr().as_u64() & !(HUGE_PAGE_SIZE as u64 - 1);
        let offset = virt.as_u64() & (HUGE_PAGE_SIZE as u64 - 1);
        return Some(PhysAddr::new(base + offset));
    }

    // Level 1: PT
    let pt = unsafe { &*(hhdm::phys_to_virt(pd_entry.addr().as_u64()) as *const PageTable) };
    let pt_entry = pt.entry(pt_index(virt));
    if !pt_entry.is_present() {
        return None;
    }

    // 4KB page
    let base = pt_entry.addr().as_u64();
    let offset = page_offset(virt) as u64;
    Some(PhysAddr::new(base + offset))
}

/// Obtém flags de mapeamento de um endereço
pub fn get_flags(virt: VirtAddr) -> Option<MapFlags> {
    let pml4_phys = read_cr3();
    let pml4 = unsafe { &*(hhdm::phys_to_virt(pml4_phys.as_u64()) as *const PageTable) };

    let pml4_entry = pml4.entry(pml4_index(virt));
    if !pml4_entry.is_present() {
        return None;
    }

    let pdpt = unsafe { &*(hhdm::phys_to_virt(pml4_entry.addr().as_u64()) as *const PageTable) };
    let pdpt_entry = pdpt.entry(pdpt_index(virt));
    if !pdpt_entry.is_present() {
        return None;
    }

    let pd = unsafe { &*(hhdm::phys_to_virt(pdpt_entry.addr().as_u64()) as *const PageTable) };
    let pd_entry = pd.entry(pd_index(virt));
    if !pd_entry.is_present() {
        return None;
    }

    let pt = unsafe { &*(hhdm::phys_to_virt(pd_entry.addr().as_u64()) as *const PageTable) };
    let pt_entry = pt.entry(pt_index(virt));
    if !pt_entry.is_present() {
        return None;
    }

    Some(pt_entry.flags())
}

/// Mapeia range de páginas
pub fn map_range(
    virt_start: VirtAddr,
    phys_start: PhysAddr,
    count: usize,
    flags: MapFlags,
) -> RmmResult<()> {
    for i in 0..count {
        let virt = virt_start + (i * PAGE_SIZE) as u64;
        let phys = phys_start + (i * PAGE_SIZE) as u64;
        map_page(virt, phys, flags)?;
    }
    Ok(())
}

/// Remove mapeamento de range de páginas
pub fn unmap_range(virt_start: VirtAddr, count: usize) -> RmmResult<()> {
    for i in 0..count {
        let virt = virt_start + (i * PAGE_SIZE) as u64;
        unmap_page(virt)?;
    }
    // Flush TLB do range inteiro
    tlb::flush_tlb_range(virt_start, virt_start + (count * PAGE_SIZE) as u64);
    Ok(())
}

// =============================================================================
// Helpers Internos
// =============================================================================

/// Obtém ou cria tabela no próximo nível
fn get_or_create_table(
    table: &mut PageTable,
    index: usize,
    flags: MapFlags,
) -> RmmResult<&mut PageTable> {
    let entry = table.entry_mut(index);

    if entry.is_present() {
        // Tabela já existe
        let phys = entry.addr();
        let virt = hhdm::phys_to_virt(phys.as_u64());
        Ok(unsafe { &mut *(virt as *mut PageTable) })
    } else {
        // Precisa criar nova tabela
        let new_table_phys = phys::alloc(FrameOwner::Kernel, Zone::Normal, AllocFlags::ZERO)
            .ok_or(RmmError::OutOfMemory)?;

        // Flags para entradas intermediárias: presente + writable + user (se aplicável)
        let intermediate_flags = if flags.is_user() {
            MapFlags::PRESENT
                .union(MapFlags::WRITABLE)
                .union(MapFlags::USER)
        } else {
            MapFlags::PRESENT.union(MapFlags::WRITABLE)
        };

        *entry = PageTableEntry::new(new_table_phys, intermediate_flags);

        let virt = hhdm::phys_to_virt(new_table_phys.as_u64());
        Ok(unsafe { &mut *(virt as *mut PageTable) })
    }
}
