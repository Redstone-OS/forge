//! Physical Memory Manager
//!
//! Gerencia frames físicos de 4 KB usando bitmap allocator.

use crate::boot_info::{BootInfo, MemoryRegion, MemoryRegionType};

/// Tamanho de um frame (4 KB)
pub const FRAME_SIZE: usize = 4096;

/// Frame físico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub number: usize,
}

impl Frame {
    /// Cria frame a partir do número
    pub const fn from_number(number: usize) -> Self {
        Self { number }
    }

    /// Obtém endereço físico do frame
    pub const fn start_address(&self) -> usize {
        self.number * FRAME_SIZE
    }

    /// Cria frame contendo o endereço
    pub fn containing_address(addr: usize) -> Self {
        Self {
            number: addr / FRAME_SIZE,
        }
    }
}

/// Gerenciador de memória física
pub struct PhysicalMemoryManager {
    /// Bitmap de frames (1 bit por frame)
    bitmap: &'static mut [u64],

    /// Número total de frames
    total_frames: usize,

    /// Número de frames livres
    free_frames: usize,
}

impl PhysicalMemoryManager {
    /// Inicializa o PMM a partir do memory map
    pub fn init(boot_info: &BootInfo) -> Self {
        // 1. Ler memory map
        let memory_regions = unsafe {
            core::slice::from_raw_parts(
                boot_info.memory_map_addr as *const MemoryRegion,
                boot_info.memory_map_size as usize,
            )
        };

        // 2. Calcular número total de frames (até 512 MB)
        let total_frames = (512 * 1024 * 1024) / FRAME_SIZE; // 131072 frames

        // 3. Alocar bitmap (1 bit por frame = 131072 bits = 16384 bytes = 4 páginas)
        let bitmap_size = (total_frames + 63) / 64; // Arredondar para u64s

        // Bitmap vai em 0x800000 (8 MB, após kernel)
        let bitmap_addr = 0x800000;
        let bitmap =
            unsafe { core::slice::from_raw_parts_mut(bitmap_addr as *mut u64, bitmap_size) };

        // 4. Inicializar bitmap (tudo ocupado)
        for entry in bitmap.iter_mut() {
            *entry = u64::MAX;
        }

        // 5. Marcar regiões usáveis como livres
        let mut free_frames = 0;
        for region in memory_regions {
            if region.region_type == MemoryRegionType::Usable {
                let start_frame = (region.base as usize) / FRAME_SIZE;
                let end_frame = ((region.base + region.length) as usize) / FRAME_SIZE;

                for frame in start_frame..end_frame {
                    // Reserve low memory (0-1MB) for safety (Video RAM, BIOS, etc)
                    if frame < 256 {
                        continue;
                    }

                    if frame < total_frames {
                        Self::mark_free_in_bitmap(bitmap, frame);
                        free_frames += 1;
                    }
                }
            }
        }

        Self {
            bitmap,
            total_frames,
            free_frames,
        }
    }

    /// Aloca um frame
    pub fn allocate(&mut self) -> Option<Frame> {
        for (i, &entry) in self.bitmap.iter().enumerate() {
            if entry != u64::MAX {
                for bit in 0..64 {
                    if (entry & (1 << bit)) == 0 {
                        let frame_number = i * 64 + bit;
                        if frame_number < self.total_frames {
                            Self::mark_used_in_bitmap(self.bitmap, frame_number);
                            self.free_frames -= 1;
                            return Some(Frame::from_number(frame_number));
                        }
                    }
                }
            }
        }
        None
    }

    /// Libera um frame
    pub fn deallocate(&mut self, frame: Frame) {
        if frame.number < self.total_frames {
            if !Self::is_free_in_bitmap(self.bitmap, frame.number) {
                Self::mark_free_in_bitmap(self.bitmap, frame.number);
                self.free_frames += 1;
            }
        }
    }

    /// Retorna estatísticas (total, livres, usados)
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            self.total_frames,
            self.free_frames,
            self.total_frames - self.free_frames,
        )
    }

    /// Verifica se frame está livre
    fn is_free_in_bitmap(bitmap: &[u64], frame: usize) -> bool {
        let index = frame / 64;
        let bit = frame % 64;
        (bitmap[index] & (1 << bit)) == 0
    }

    /// Marca frame como livre
    fn mark_free_in_bitmap(bitmap: &mut [u64], frame: usize) {
        let index = frame / 64;
        let bit = frame % 64;
        bitmap[index] &= !(1 << bit);
    }

    /// Marca frame como usado
    fn mark_used_in_bitmap(bitmap: &mut [u64], frame: usize) {
        let index = frame / 64;
        let bit = frame % 64;
        bitmap[index] |= 1 << bit;
    }
}
