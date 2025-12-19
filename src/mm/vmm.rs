//! Gerenciador de Memória Virtual (VMM)
//!
//! Implementa paginação de 4 níveis (PML4 → PDPT → PD → PT) para x86_64.
//! Permite mapear endereços virtuais para físicos.

use crate::mm::pmm::{Frame, PhysicalMemoryManager};

/// Flags de entrada de tabela de páginas
pub mod flags {
    /// Página presente na memória
    pub const PRESENT: u64 = 1 << 0;
    /// Página pode ser escrita
    pub const WRITABLE: u64 = 1 << 1;
    /// Página acessível em modo usuário
    pub const USER: u64 = 1 << 2;
    /// Write-through caching
    pub const WRITE_THROUGH: u64 = 1 << 3;
    /// Cache desabilitado
    pub const NO_CACHE: u64 = 1 << 4;
    /// Página foi acessada
    pub const ACCESSED: u64 = 1 << 5;
    /// Página foi modificada (dirty)
    pub const DIRTY: u64 = 1 << 6;
    /// Página grande (2MB ou 1GB)
    pub const HUGE: u64 = 1 << 7;
    /// Não executável (NX bit)
    pub const NO_EXECUTE: u64 = 1 << 63;
}

/// Entrada de tabela de páginas (PTE)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    /// Cria nova entrada com endereço físico e flags
    pub const fn new(phys_addr: u64, flags: u64) -> Self {
        Self {
            entry: (phys_addr & 0x000F_FFFF_FFFF_F000) | flags,
        }
    }

    /// Cria entrada vazia (não presente)
    pub const fn empty() -> Self {
        Self { entry: 0 }
    }

    /// Verifica se a página está presente
    pub fn is_present(&self) -> bool {
        self.entry & flags::PRESENT != 0
    }

    /// Obtém endereço físico da entrada
    pub fn phys_addr(&self) -> u64 {
        self.entry & 0x000F_FFFF_FFFF_F000
    }

    /// Define flags da entrada
    pub fn set_flags(&mut self, flags: u64) {
        self.entry = (self.entry & 0x000F_FFFF_FFFF_F000) | flags;
    }

    /// Zera a entrada
    pub fn clear(&mut self) {
        self.entry = 0;
    }
}

/// Tabela de páginas (512 entradas)
#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    /// Cria nova tabela zerada
    pub fn new() -> Self {
        Self {
            entries: [PageTableEntry::empty(); 512],
        }
    }

    /// Zera todas as entradas
    pub fn zero(&mut self) {
        for entry in &mut self.entries {
            entry.clear();
        }
    }

    /// Obtém referência mutável para entrada no índice
    pub fn entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }

    /// Obtém referência para entrada no índice
    pub fn entry(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }
}

/// Gerenciador de Memória Virtual
pub struct VirtualMemoryManager {
    pml4: &'static mut PageTable,
}

impl VirtualMemoryManager {
    /// Inicializa o VMM criando PML4
    pub fn init(pmm: &mut PhysicalMemoryManager) -> Self {
        crate::drivers::legacy::serial::println("[VMM] Allocating PML4...");
        // Alocar frame para PML4
        let pml4_frame = pmm.allocate().expect("Sem memória para PML4");

        let pml4_addr = pml4_frame.start_address();
        crate::drivers::legacy::serial::print("[VMM] PML4 allocated at 0x");
        crate::drivers::legacy::serial::print_hex(pml4_addr);
        crate::drivers::legacy::serial::println("");

        let pml4_ptr = pml4_addr as *mut PageTable;

        // Zerar PML4
        crate::drivers::legacy::serial::println("[VMM] Zeroing PML4...");
        let pml4 = unsafe {
            pml4_ptr.write(PageTable::new());
            &mut *pml4_ptr
        };
        crate::drivers::legacy::serial::println("[VMM] PML4 zeroed.");

        Self { pml4 }
    }

    /// Obtém ou cria tabela de páginas no índice especificado
    fn get_or_create_table(
        table: &mut PageTable,
        index: usize,
        pmm: &mut PhysicalMemoryManager,
    ) -> Result<&'static mut PageTable, &'static str> {
        let entry = table.entry_mut(index);

        if !entry.is_present() {
            // Alocar nova tabela
            let frame = pmm.allocate().ok_or("Sem memória para tabela de páginas")?;
            let phys_addr = frame.start_address();

            // Configurar entrada
            *entry = PageTableEntry::new(phys_addr as u64, flags::PRESENT | flags::WRITABLE);

            // Zerar nova tabela
            // TODO: Adicionar testes de mapeamento após resolver linking
            let new_table = unsafe { &mut *(phys_addr as *mut PageTable) };
            new_table.zero();
        }

        let phys_addr = entry.phys_addr();
        Ok(unsafe { &mut *(phys_addr as *mut PageTable) })
    }

    /// Mapeia página virtual para física
    pub fn map_page(
        &mut self,
        virt_addr: u64,
        phys_addr: u64,
        flags: u64,
        pmm: &mut PhysicalMemoryManager,
    ) -> Result<(), &'static str> {
        // Extrair índices do endereço virtual
        let pml4_index = ((virt_addr >> 39) & 0x1FF) as usize;
        let pdpt_index = ((virt_addr >> 30) & 0x1FF) as usize;
        let pd_index = ((virt_addr >> 21) & 0x1FF) as usize;
        let pt_index = ((virt_addr >> 12) & 0x1FF) as usize;

        // Caminhar pelas tabelas, criando se necessário
        let pdpt = Self::get_or_create_table(self.pml4, pml4_index, pmm)?;
        let pd = Self::get_or_create_table(pdpt, pdpt_index, pmm)?;
        let pt = Self::get_or_create_table(pd, pd_index, pmm)?;

        // Mapear a página
        let entry = pt.entry_mut(pt_index);
        *entry = PageTableEntry::new(phys_addr, flags | flags::PRESENT);

        // Invalidar TLB para este endereço
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) virt_addr, options(nostack, preserves_flags));
        }

        Ok(())
    }

    /// Desmapeia página virtual
    pub fn unmap_page(&mut self, virt_addr: u64) -> Result<(), &'static str> {
        // Extrair índices
        let pml4_index = ((virt_addr >> 39) & 0x1FF) as usize;
        let pdpt_index = ((virt_addr >> 30) & 0x1FF) as usize;
        let pd_index = ((virt_addr >> 21) & 0x1FF) as usize;
        let pt_index = ((virt_addr >> 12) & 0x1FF) as usize;

        // Verificar se as tabelas existem
        let pml4_entry = self.pml4.entry(pml4_index);
        if !pml4_entry.is_present() {
            return Err("Página não mapeada");
        }

        let pdpt = unsafe { &mut *(pml4_entry.phys_addr() as *mut PageTable) };
        let pdpt_entry = pdpt.entry(pdpt_index);
        if !pdpt_entry.is_present() {
            return Err("Página não mapeada");
        }

        let pd = unsafe { &mut *(pdpt_entry.phys_addr() as *mut PageTable) };
        let pd_entry = pd.entry(pd_index);
        if !pd_entry.is_present() {
            return Err("Página não mapeada");
        }

        let pt = unsafe { &mut *(pd_entry.phys_addr() as *mut PageTable) };

        // Zerar entrada
        pt.entry_mut(pt_index).clear();

        // Invalidar TLB
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) virt_addr, options(nostack, preserves_flags));
        }

        Ok(())
    }

    /// Mapeia região de memória identicamente (virtual = físico)
    pub fn identity_map(
        &mut self,
        start: u64,
        end: u64,
        flags: u64,
        pmm: &mut PhysicalMemoryManager,
    ) -> Result<(), &'static str> {
        // Alinhar para páginas de 4KB
        let start_page = start & !0xFFF;
        let end_page = (end + 0xFFF) & !0xFFF;

        // Mapear cada página
        let mut addr = start_page;
        while addr < end_page {
            self.map_page(addr, addr, flags, pmm)?;
            addr += 4096;
        }

        Ok(())
    }

    // TODO: Reativar após resolver problema de linking
    // Funções de debug removidas temporariamente para reduzir tamanho do binário

    /// Ativa o VMM carregando PML4 no CR3
    pub fn activate(&self) {
        let pml4_phys = self.pml4 as *const _ as u64;
        unsafe {
            core::arch::asm!("mov cr3, {}", in(reg) pml4_phys, options(nostack, preserves_flags));
        }
    }
}
