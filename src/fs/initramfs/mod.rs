#![allow(dead_code)]
//! InitramFS - filesystem em memória do boot

use crate::fs::vfs::inode::{DirEntry, FsError, InodeOps};
use crate::mm::VirtAddr;
use crate::sync::Spinlock;
use alloc::vec::Vec;
use core::slice;

/// Armazenamento global do Initramfs (Raw Bytes)
static INITRAMFS_DATA: Spinlock<Option<&'static [u8]>> = Spinlock::new(None);

/// Header USTAR (Tamanho fixo 512 bytes)
const TAR_BLOCK_SIZE: usize = 512;
const TAR_NAME_OFFSET: usize = 0;
const TAR_NAME_LEN: usize = 100;
const TAR_SIZE_OFFSET: usize = 124;
const TAR_SIZE_LEN: usize = 12;
const TAR_TYPE_OFFSET: usize = 156;
const TAR_MAGIC_OFFSET: usize = 257;

/// Helper para parsear octal
fn parse_octal(data: &[u8]) -> usize {
    let mut size = 0;
    for &byte in data {
        if byte < b'0' || byte > b'7' {
            break;
        }
        size = size * 8 + (byte - b'0') as usize;
    }
    size
}

/// Helper para alinhar para próximo bloco de 512 bytes
fn align_up_512(size: usize) -> usize {
    (size + TAR_BLOCK_SIZE - 1) & !(TAR_BLOCK_SIZE - 1)
}

/// Inode do initramfs
struct InitramfsInode {
    data: *const u8,
    size: usize,
}

impl InodeOps for InitramfsInode {
    fn lookup(&self, _name: &str) -> Option<u64> {
        // TODO: implementar lookup via VFS inode system
        None
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, FsError> {
        let offset = offset as usize;
        if offset >= self.size {
            return Ok(0);
        }

        let to_read = buf.len().min(self.size - offset);
        unsafe {
            core::ptr::copy_nonoverlapping(self.data.add(offset), buf.as_mut_ptr(), to_read);
        }
        Ok(to_read)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize, FsError> {
        Err(FsError::ReadOnly)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, FsError> {
        Ok(Vec::new())
    }
}

unsafe impl Sync for InitramfsInode {}
unsafe impl Send for InitramfsInode {}

/// Carrega initramfs da memória
pub fn init(addr: VirtAddr, size: usize) {
    crate::kinfo!("(InitramFS) Carregando de addr=", addr.as_u64());
    crate::kinfo!("(InitramFS) Tamanho:", size as u64);

    // SAFETY: O bootloader garante que esta memória é válida e contém o initramfs
    let data = unsafe { slice::from_raw_parts(addr.as_ptr(), size) };
    *INITRAMFS_DATA.lock() = Some(data);
}

/// Busca um arquivo no initramfs e retorna seus dados
/// Usado diretamente pelo spawn() enquanto VFS não está pronto
pub fn lookup_file(path: &str) -> Option<&'static [u8]> {
    let guard = INITRAMFS_DATA.lock();
    let data = (*guard)?;

    // Normalizar path: remover leading slashes
    let path_bytes = path.as_bytes();
    let mut start = 0;
    while start < path_bytes.len() && (path_bytes[start] == b'/' || path_bytes[start] == b'.') {
        start += 1;
    }
    let search_bytes = &path_bytes[start..];

    // Log do path sendo buscado (primeiros 4 bytes para debug)
    if search_bytes.len() >= 4 {
        crate::ktrace!("(InitramFS) Search[0]:", search_bytes[0] as u64);
        crate::ktrace!("(InitramFS) Search[1]:", search_bytes[1] as u64);
        crate::ktrace!("(InitramFS) Search[2]:", search_bytes[2] as u64);
        crate::ktrace!("(InitramFS) Search[3]:", search_bytes[3] as u64);
    }

    crate::ktrace!("(InitramFS) Search Loop Start");

    let mut offset = 0;
    while offset + TAR_BLOCK_SIZE <= data.len() {
        let header = &data[offset..offset + TAR_BLOCK_SIZE];

        // Verificar magic "ustar" (com ou sem null terminator, ou espaço)
        if &header[TAR_MAGIC_OFFSET..TAR_MAGIC_OFFSET + 5] != b"ustar" {
            break;
        }

        // Ler tamanho primeiro para validar offset
        let size = parse_octal(&header[TAR_SIZE_OFFSET..TAR_SIZE_OFFSET + TAR_SIZE_LEN]);

        // Ler nome (com limite seguro) - MANUAL BYTES ONLY
        let name_bytes = &header[TAR_NAME_OFFSET..TAR_NAME_OFFSET + TAR_NAME_LEN];

        // Calcular tamanho manualmente
        let mut name_len = 0;
        while name_len < TAR_NAME_LEN {
            if name_bytes[name_len] == 0 {
                break;
            }
            name_len += 1;
        }
        let name_slice = &name_bytes[..name_len];

        // Comparacao manual com "init" para debug (byte a byte)
        if name_len >= 4 {
            for i in 0..=name_len - 4 {
                if &name_slice[i..i + 4] == b"init" {
                    crate::ktrace!("(TAR) Check: (found init substring)");
                    break;
                }
            }
        }

        // Normalização MANUAL de bytes (remove ./ e /)
        let mut start_idx = 0;
        while start_idx < name_len {
            let c = name_slice[start_idx];
            if c == b'.' || c == b'/' {
                start_idx += 1;
            } else {
                break;
            }
        }
        let normalized_bytes = &name_slice[start_idx..];

        let is_match = normalized_bytes == search_bytes;

        // Tipo '0' ou '\0' é arquivo normal
        let type_flag = header[TAR_TYPE_OFFSET];
        let is_file = type_flag == b'0' || type_flag == 0;

        if is_file && is_match {
            // Log do arquivo encontrado
            if normalized_bytes.len() >= 4 {
                crate::ktrace!("(TAR) Match file[0]:", normalized_bytes[0] as u64);
                crate::ktrace!("(TAR) Match file[1]:", normalized_bytes[1] as u64);
                crate::ktrace!("(TAR) Match file[2]:", normalized_bytes[2] as u64);
                crate::ktrace!("(TAR) Match file[3]:", normalized_bytes[3] as u64);
            }
            crate::ktrace!("(TAR) Matched! Size:", size as u64);

            let file_start = offset + TAR_BLOCK_SIZE;

            // Segurança: Verificar limites antes de criar slice
            if file_start
                .checked_add(size)
                .map_or(true, |end| end > data.len())
            {
                crate::kerror!("(InitramFS) Arquivo truncado ou overflow");
                return None;
            }

            crate::ktrace!("(TAR) Creating slice...");
            // Usando from_raw_parts para evitar problemas com lifetimes/bounds check implícitos
            let ptr = unsafe { data.as_ptr().add(file_start) };
            let slice = unsafe { slice::from_raw_parts(ptr, size) };

            crate::ktrace!("(TAR) Returning slice len:", slice.len() as u64);
            return Some(slice);
        }

        // Avançar para próximo header
        let file_block_size = align_up_512(size);

        match offset
            .checked_add(TAR_BLOCK_SIZE)
            .and_then(|o| o.checked_add(file_block_size))
        {
            Some(new_offset) => {
                if new_offset <= offset {
                    crate::kerror!("(InitramFS) Loop infinito detectado!");
                    break;
                }
                offset = new_offset;
            }
            None => {
                crate::kerror!("(InitramFS) Overflow de offset!");
                break;
            }
        }
    }

    crate::ktrace!("(InitramFS) Arquivo não encontrado.");
    None
}
