use super::frame::PhysFrame;
use super::stats::PmmStats;
use crate::core::handoff::{BootInfo, MemoryType};
use crate::mm::addr::{self, PhysAddr};
use crate::mm::config::PAGE_SIZE;
use crate::mm::ops::memops;
use core::sync::atomic::Ordering;

/// BitmapFrameAllocator
///
/// Gerencia memória física usando um bitmap.
/// Protegido por Mutex externo (em mod.rs).
pub struct BitmapFrameAllocator {
    /// Início da memória gerenciada (geralmente 0)
    _memory_base: PhysAddr,
    /// Ponteiro para o bitmap (em HHDM virtual address)
    bitmap_ptr: *mut u64,
    /// Tamanho do bitmap em u64
    bitmap_len: usize,
    /// Total de frames gerenciados
    total_frames: usize,
    /// Dica para próxima alocação
    next_free: usize,
    /// Estatísticas
    stats: PmmStats,
}

// SAFETY: Send/Sync seguro pois o acesso é via Mutex externo
unsafe impl Send for BitmapFrameAllocator {}
unsafe impl Sync for BitmapFrameAllocator {}

impl BitmapFrameAllocator {
    /// Cria um alocador vazio
    pub const fn empty() -> Self {
        Self {
            _memory_base: PhysAddr::new(0),
            bitmap_ptr: core::ptr::null_mut(),
            bitmap_len: 0,
            total_frames: 0,
            next_free: 0,
            stats: PmmStats::new(),
        }
    }

    /// Inicializa o alocador
    pub unsafe fn init(&mut self, boot_info: &'static BootInfo) {
        crate::kinfo!("(PMM) Inicializando BitmapFrameAllocator...");

        // 1. Calcular memória total e encontrar região para o bitmap
        crate::kdebug!("(PMM) Passo 1: Escaneando memory map...");
        let (max_phys, _usable_regions) = self.scan_memory_map(boot_info);
        crate::kdebug!(
            "(PMM) max_phys={:#x} ({} MB)",
            max_phys.as_u64(),
            max_phys.as_u64() / (1024 * 1024)
        );

        // 2. Calcular tamanho do bitmap
        crate::kdebug!("(PMM) Passo 2: Calculando tamanho do bitmap...");
        self.total_frames = max_phys.as_usize() / PAGE_SIZE;
        self.stats.total_frames = self.total_frames;

        let bitmap_size_bytes = (self.total_frames + 7) / 8;
        let bitmap_size_u64 = (bitmap_size_bytes + 7) / 8;
        self.bitmap_len = bitmap_size_u64;
        crate::kdebug!(
            "(PMM) total_frames={}, bitmap_size={}KB",
            self.total_frames,
            (bitmap_size_u64 * 8) / 1024
        );

        // 3. Alocar região física para o bitmap
        crate::kdebug!("(PMM) Passo 3: Encontrando região para bitmap...");
        let bitmap_phys = self.find_bitmap_region(boot_info, bitmap_size_u64 * 8);
        crate::kdebug!("(PMM) bitmap_phys={:#x}", bitmap_phys.as_u64());

        // 4. Mapear (HHDM) e limpar bitmap
        crate::kdebug!("(PMM) Passo 4: Convertendo phys_to_virt...");
        self.bitmap_ptr = addr::phys_to_virt(bitmap_phys).as_mut_ptr();
        crate::kdebug!(
            "(PMM) Bitmap em phys={:?} virt={:p} size={}KB",
            bitmap_phys,
            self.bitmap_ptr,
            (self.bitmap_len * 8) / 1024
        );

        // Memset 0xFF (tudo ocupado inicialmente)
        crate::kdebug!("(PMM) Passo 5: memset bitmap...");

        // Debug: Testar escrita de um único byte primeiro
        crate::kdebug!(
            "(PMM) Testando escrita de 1 byte em {:p}...",
            self.bitmap_ptr
        );
        core::ptr::write_volatile(self.bitmap_ptr as *mut u8, 0xFF);
        crate::kdebug!("(PMM) Escrita de 1 byte OK!");

        // Memset manual para evitar possível problema com otimização
        let ptr = self.bitmap_ptr as *mut u8;
        let len = self.bitmap_len * 8;
        crate::kdebug!("(PMM) Iniciando memset de {} bytes...", len);
        for i in 0..len {
            core::ptr::write_volatile(ptr.add(i), 0xFF);
        }
        crate::kdebug!("(PMM) memset completo!");

        // 5. Liberar regiões usable
        self.init_free_regions(boot_info, bitmap_phys, (bitmap_size_u64 * 8) as u64);

        crate::kinfo!(
            "(PMM) Init completo. Total: {} frames. Livres: {}",
            self.total_frames,
            self.total_frames - self.stats.used_frames.load(Ordering::Relaxed)
        );
    }

    /// Aloca um frame físico
    pub fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let start_search = self.next_free;
        let mut i = 0;

        while i < self.bitmap_len {
            let idx = (start_search + i) % self.bitmap_len;

            unsafe {
                let entry_ptr = self.bitmap_ptr.add(idx);
                let entry = *entry_ptr;

                if entry != u64::MAX {
                    let bit = entry.trailing_ones() as usize;
                    let frame_idx = idx * 64 + bit;

                    if frame_idx < self.total_frames {
                        // Marcar ocupado
                        *entry_ptr |= 1 << bit;

                        self.stats.inc_alloc();
                        self.next_free = idx;

                        return Some(PhysFrame::from_start_address(PhysAddr::new(
                            frame_idx as u64 * PAGE_SIZE as u64,
                        )));
                    }
                }
            }
            i += 1;
        }

        None
    }

    /// Desaloca um frame físico
    pub fn deallocate_frame(&mut self, frame: PhysFrame) {
        let frame_idx = (frame.start_address().as_u64() / PAGE_SIZE as u64) as usize;

        if frame_idx >= self.total_frames {
            crate::kwarn!(
                "(PMM) Tentativa de desalocar frame fora do range: {}",
                frame_idx
            );
            return;
        }

        let idx = frame_idx / 64;
        let bit = frame_idx % 64;

        unsafe {
            let entry_ptr = self.bitmap_ptr.add(idx);
            let entry = *entry_ptr;

            if (entry & (1 << bit)) == 0 {
                crate::kwarn!("(PMM) Double-free detectado no frame {}", frame_idx);
                return;
            }

            *entry_ptr &= !(1 << bit);
        }

        self.stats.inc_free();
        if idx < self.next_free {
            self.next_free = idx;
        }
    }

    fn scan_memory_map(&self, boot_info: &BootInfo) -> (PhysAddr, usize) {
        let mut max_phys = 0;
        let mut count = 0;

        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = boot_info.memory_map_len as usize;

        for i in 0..map_len {
            unsafe {
                let entry = &*map_ptr.add(i);
                if entry.typ == MemoryType::Usable {
                    let end = entry.base + entry.len;
                    if end > max_phys {
                        max_phys = end;
                    }
                    count += 1;
                }
            }
        }
        (PhysAddr::new(max_phys), count)
    }

    fn find_bitmap_region(&self, boot_info: &BootInfo, size_bytes: usize) -> PhysAddr {
        // O bootloader mapeia os primeiros 4GB com huge pages de 2MB.
        // Quando mapeia o kernel, ele QUEBRA a huge page que contém o kernel,
        // criando páginas de 4KB apenas para o kernel, deixando as outras não mapeadas.
        //
        // Solução: alocar bitmap na PRÓXIMA huge page INTACTA após o kernel.
        const HUGE_PAGE_SIZE: u64 = 2 * 1024 * 1024; // 2MB

        let kernel_end = boot_info.kernel_phys_addr + boot_info.kernel_size;

        // Alinhar ao próximo limite de 2MB (próxima huge page intacta)
        let next_huge_page = (kernel_end + HUGE_PAGE_SIZE - 1) & !(HUGE_PAGE_SIZE - 1);

        crate::kdebug!(
            "(PMM) kernel_end={:#x}, próxima huge page intacta={:#x}",
            kernel_end,
            next_huge_page
        );

        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = boot_info.memory_map_len as usize;

        for i in 0..map_len {
            unsafe {
                let entry = &*map_ptr.add(i);
                if entry.typ == MemoryType::Usable {
                    let entry_start = entry.base;
                    let entry_end = entry.base + entry.len;

                    // Região deve conter espaço APÓS next_huge_page
                    if entry_end > next_huge_page {
                        // Calcular início efetivo dentro desta região
                        let effective_start = core::cmp::max(entry_start, next_huge_page);

                        // Verificar se há espaço suficiente
                        if entry_end >= effective_start + size_bytes as u64 {
                            // Alinhar
                            let bitmap_start =
                                (effective_start + PAGE_SIZE as u64 - 1) & !(PAGE_SIZE as u64 - 1);

                            if bitmap_start + size_bytes as u64 <= entry_end {
                                crate::kdebug!(
                                    "(PMM) Encontrada região para bitmap: {:#x} (após kernel)",
                                    bitmap_start
                                );
                                return PhysAddr::new(bitmap_start);
                            }
                        }
                    }
                }
            }
        }
        panic!("(PMM) Falha ao alocar memória para o bitmap!");
    }

    unsafe fn init_free_regions(
        &mut self,
        boot_info: &BootInfo,
        bitmap_start: PhysAddr,
        bitmap_size: u64,
    ) {
        let kernel_start = boot_info.kernel_phys_addr;
        let kernel_end = kernel_start + boot_info.kernel_size;
        let bitmap_end = bitmap_start.as_u64() + bitmap_size;

        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = boot_info.memory_map_len as usize;

        for i in 0..map_len {
            let entry = &*map_ptr.add(i);
            if entry.typ == MemoryType::Usable {
                let start_frame = entry.base / PAGE_SIZE as u64;
                let end_frame = (entry.base + entry.len) / PAGE_SIZE as u64;

                for f in start_frame..end_frame {
                    let addr = f * PAGE_SIZE as u64;
                    // Proteções
                    if addr < 0x100000 {
                        continue;
                    } // Primeiros 1MB
                    if addr >= kernel_start && addr < kernel_end {
                        continue;
                    }
                    if addr >= bitmap_start.as_u64() && addr < bitmap_end {
                        continue;
                    }

                    self.deallocate_frame_internal(f as usize);
                }
            }
        }
    }

    fn deallocate_frame_internal(&mut self, frame_idx: usize) {
        if frame_idx >= self.total_frames {
            return;
        }

        let idx = frame_idx / 64;
        let bit = frame_idx % 64;

        unsafe {
            let ptr = self.bitmap_ptr.add(idx);
            *ptr &= !(1 << bit);
        }
        self.stats.used_frames.fetch_sub(1, Ordering::Relaxed);
    }
}
