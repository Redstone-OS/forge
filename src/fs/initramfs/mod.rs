//! InitramFS - filesystem em memória do boot

use crate::fs::vfs::inode::{DirEntry, FsError, InodeOps};
use crate::mm::VirtAddr;
use crate::sync::Spinlock;
use alloc::vec::Vec;
use core::slice;
use core::str;

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

    // Remover leading slash e ./ para comparar
    let search_path = path.trim_start_matches('/').trim_start_matches("./");

    let mut offset = 0;
    while offset + TAR_BLOCK_SIZE <= data.len() {
        let header = &data[offset..offset + TAR_BLOCK_SIZE];

        // Verificar magic "ustar" (com ou sem null terminator, ou espaço)
        // Alguns TARs usam "ustar\0" ou "ustar "
        if &header[TAR_MAGIC_OFFSET..TAR_MAGIC_OFFSET + 5] != b"ustar" {
            // Fim do arquivo ou bloco inválido
            break;
        }

        // Ler nome
        let name_bytes = &header[TAR_NAME_OFFSET..TAR_NAME_OFFSET + TAR_NAME_LEN];
        let name_len = name_bytes
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(TAR_NAME_LEN);
        let name = str::from_utf8(&name_bytes[..name_len]).unwrap_or("");

        // Normalizar nome do arquivo no TAR
        let normalized_name = name.trim_start_matches("./").trim_start_matches('/');

        // Ler tamanho
        let size = parse_octal(&header[TAR_SIZE_OFFSET..TAR_SIZE_OFFSET + TAR_SIZE_LEN]);

        // Tipo '0' ou '\0' é arquivo normal
        let type_flag = header[TAR_TYPE_OFFSET];
        let is_file = type_flag == b'0' || type_flag == 0;

        crate::ktrace!("(TAR) Found:", name);

        if is_file && normalized_name == search_path {
            crate::ktrace!("(TAR) Matched! Size:", size as u64);
            // Encontrado!
            let file_start = offset + TAR_BLOCK_SIZE;
            if file_start + size > data.len() {
                crate::kerror!("(InitramFS) Arquivo truncado:", name);
                return None;
            }

            crate::ktrace!("(TAR) Creating slice...");
            // Retornar slice static (seguro pois initramfs vive pra sempre na RAM)
            // let file_data = &data[file_start..file_start + size];

            // Usando from_raw_parts para evitar problemas com lifetimes/bounds check implícitos
            let ptr = unsafe { data.as_ptr().add(file_start) };
            let slice = unsafe { slice::from_raw_parts(ptr, size) };

            crate::ktrace!("(TAR) Returning slice len:", slice.len() as u64);

            // Transmutar lifetime para 'static pois os dados são persistentes na RAM
            return Some(slice);
        }

        // Avançar para próximo header
        offset += TAR_BLOCK_SIZE + align_up_512(size);
    }

    None
}
