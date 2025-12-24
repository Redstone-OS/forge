//! Physical Memory Manager (PMM).
//!
//! Gerencia a alocação de frames físicos (páginas de 4KiB) usando um Bitmap.
//! Simples, eficiente e suficiente para o Kernel.

use crate::core::handoff::{BootInfo, MemoryType};
use crate::sync::Mutex;
use core::sync::atomic::{AtomicUsize, Ordering};

pub const FRAME_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysFrame {
    pub addr: u64,
}

impl PhysFrame {
    pub fn containing_address(addr: u64) -> Self {
        Self {
            addr: addr - (addr % FRAME_SIZE as u64),
        }
    }
}

/// O alocador global de frames físicos.
pub static FRAME_ALLOCATOR: Mutex<BitmapFrameAllocator> = Mutex::new(BitmapFrameAllocator::empty());

pub struct BitmapFrameAllocator {
    /// Início da região de memória gerenciada pelo bitmap.
    memory_base: u64,
    /// Bitmap onde cada bit representa um frame de 4KiB.
    /// 0 = Livre, 1 = Usado.
    bitmap: &'static mut [u64],
    /// Total de frames gerenciados.
    total_frames: usize,
    /// Dica para a próxima alocação (round-robin simples).
    next_free: usize,
    /// Estatística de uso.
    used_frames: usize,
}

impl BitmapFrameAllocator {
    const fn empty() -> Self {
        Self {
            memory_base: 0,
            bitmap: &mut [],
            total_frames: 0,
            next_free: 0,
            used_frames: 0,
        }
    }

    /// Inicializa o alocador usando o BootInfo.
    /// O Kernel deve chamar isso bem cedo.
    ///
    /// # Safety
    /// O BootInfo deve conter um mapa de memória válido e não sobreposto.
    pub unsafe fn init(&mut self, boot_info: &'static BootInfo) {
        // Validação crítica: Memory map deve existir
        if boot_info.memory_map_len == 0 {
            panic!("BootInfo não contém mapa de memória válido!");
        }

        crate::kinfo!("PMM: Validando memory map...");
        let map_ptr = boot_info.memory_map_addr as *const crate::core::handoff::MemoryMapEntry;
        let map_len = boot_info.memory_map_len as usize;

        crate::kinfo!("PMM: Criando slice ({} entradas)...", map_len);
        let regions = core::slice::from_raw_parts(map_ptr, map_len);

        // 1. Calcular memória total APENAS da RAM utilizável (ignora MMIO)
        crate::kinfo!("PMM: Calculando memória total...");

        let mut max_phys_addr = 0;
        for region in regions {
            // Usar APENAS ConventionalMemory para definir tamanho do bitmap
            // Isso ignora MMIO (devices) que ficam em endereços altíssimos
            if region.typ == crate::core::handoff::MemoryType::Usable {
                let end = region.base + region.len;
                if end > max_phys_addr {
                    max_phys_addr = end;
                }
            }
        }

        crate::kinfo!(
            "PMM: Memória máxima: {:#x} ({} MB)",
            max_phys_addr,
            max_phys_addr / 1024 / 1024
        );

        // 2. Definir onde o bitmap vai ficar.
        crate::kinfo!("PMM: Alocando bitmap...");
        let kernel_end = boot_info.kernel_phys_addr + boot_info.kernel_size;
        let bitmap_phys_addr = (kernel_end + FRAME_SIZE as u64 - 1) & !(FRAME_SIZE as u64 - 1);

        // Tamanho do bitmap
        let total_frames = (max_phys_addr as usize) / FRAME_SIZE;
        let bitmap_size_bytes = (total_frames + 7) / 8;
        let bitmap_size_u64 = (bitmap_size_bytes + 7) / 8;

        crate::kinfo!(
            "PMM: Bitmap em {:#x}, {} frames",
            bitmap_phys_addr,
            total_frames
        );

        // CRÍTICO: Mapear bitmap antes de usar!
        let bitmap_ptr = bitmap_phys_addr as *mut u64;
        self.bitmap = core::slice::from_raw_parts_mut(bitmap_ptr, bitmap_size_u64);

        crate::kinfo!("PMM: Zerando bitmap...");
        self.bitmap.fill(u64::MAX); // Marcar tudo como ocupado

        self.memory_base = 0;
        self.total_frames = total_frames;
        self.used_frames = total_frames;

        // 3. Liberar regiões usáveis
        crate::kinfo!("PMM: Liberando regiões usáveis...");
        let bitmap_end = bitmap_phys_addr + (bitmap_size_u64 * 8) as u64;

        for region in regions {
            if region.typ == crate::core::handoff::MemoryType::Usable {
                let start_frame = region.base / FRAME_SIZE as u64;
                let end_frame = (region.base + region.len) / FRAME_SIZE as u64;

                for frame_idx in start_frame..end_frame {
                    let addr = frame_idx * FRAME_SIZE as u64;

                    if addr >= boot_info.kernel_phys_addr && addr < bitmap_end {
                        continue;
                    }

                    if addr == 0 {
                        continue;
                    }

                    if frame_idx < total_frames as u64 {
                        self.deallocate_frame(frame_idx as usize);
                    }
                }
            }
        }

        crate::kinfo!(
            "PMM Initialized. Total Frames: {}, Free: {}",
            self.total_frames,
            self.total_frames - self.used_frames
        );
    }

    pub fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // Busca linear simples com "next_free" optimization
        let start_search = self.next_free;

        for i in 0..self.bitmap.len() {
            let idx = (start_search + i) % self.bitmap.len();
            let entry = self.bitmap[idx];

            if entry != u64::MAX {
                // Se não está tudo cheio (todos 1s)
                // Encontrar o bit 0 (livre)
                let bit = entry.trailing_ones() as usize; // Retorna quantos 1s seguidos no final (LSB)
                                                          // Se entry não é MAX, trailing_ones < 64.

                let frame_idx = idx * 64 + bit;

                if frame_idx < self.total_frames {
                    // Marcar como usado
                    self.bitmap[idx] |= 1 << bit;
                    self.used_frames += 1;
                    self.next_free = idx;

                    return Some(PhysFrame {
                        addr: frame_idx as u64 * FRAME_SIZE as u64,
                    });
                }
            }
        }

        None // OOM (Out of Memory)
    }

    pub fn deallocate_frame(&mut self, frame_idx: usize) {
        if frame_idx >= self.total_frames {
            return;
        }

        let idx = frame_idx / 64;
        let bit = frame_idx % 64;

        // Verificar se já estava livre (Double Free)
        if (self.bitmap[idx] & (1 << bit)) == 0 {
            // Log warning? Panic? Por enquanto ignora.
            return;
        }

        // Marcar como livre (0)
        self.bitmap[idx] &= !(1 << bit);
        self.used_frames -= 1;

        // Otimização: se liberamos algo antes do next_free, atualizamos
        if idx < self.next_free {
            self.next_free = idx;
        }
    }
}
