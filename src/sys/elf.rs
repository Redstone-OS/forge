//! Loader de Executáveis ELF64
//!
//! Parseia binário ELF, aloca memória e prepara entry point.

use crate::mm::pmm::FRAME_ALLOCATOR;
use crate::mm::vmm::{self, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE};
use crate::sys::Errno;
use core::mem::size_of;

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const PT_LOAD: u32 = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ElfHeader {
    magic: [u8; 4],
    class: u8,
    data: u8,
    version: u8,
    osabi: u8,
    abiversion: u8,
    pad: [u8; 7],
    typ: u16,
    machine: u16,
    version2: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ProgramHeader {
    typ: u32,
    flags: u32,
    offset: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    align: u64,
}

/// Carrega binário ELF no espaço de endereçamento atual.
/// Retorna entry point ou erro.
pub unsafe fn load(data: &[u8]) -> Result<u64, Errno> {
    if data.len() < size_of::<ElfHeader>() {
        return Err(Errno::ENOEXEC);
    }

    let header = &*(data.as_ptr() as *const ElfHeader);

    if header.magic != ELF_MAGIC || header.class != 2 {
        return Err(Errno::ENOEXEC);
    }

    let ph_offset = header.phoff as usize;
    let ph_size = header.phentsize as usize;
    let ph_count = header.phnum as usize;

    for i in 0..ph_count {
        let offset = ph_offset + i * ph_size;
        if offset + size_of::<ProgramHeader>() > data.len() {
            return Err(Errno::ENOEXEC);
        }

        let ph = &*(data.as_ptr().add(offset) as *const ProgramHeader);

        if ph.typ == PT_LOAD && ph.memsz > 0 {
            let page_flags = PAGE_PRESENT | PAGE_USER | PAGE_WRITABLE;

            let start_addr = ph.vaddr;
            let end_addr = start_addr + ph.memsz;
            let start_page = start_addr & !0xFFF;
            let end_page = (end_addr + 0xFFF) & !0xFFF;

            let mut curr = start_page;
            while curr < end_page {
                if let Some((_phys, flags)) = vmm::translate_addr_with_flags(curr) {
                    if flags & vmm::PAGE_USER != 0 {
                        curr += 4096;
                        continue;
                    }
                }

                let frame = FRAME_ALLOCATOR
                    .lock()
                    .allocate_frame()
                    .ok_or(Errno::ENOMEM)?;

                if let Err(_) = vmm::map_page(curr, frame.addr(), page_flags) {
                    return Err(Errno::ENOMEM);
                }

                core::arch::asm!("invlpg [{0}]", in(reg) curr, options(nostack, preserves_flags));

                let ptr = curr as *mut u8;
                for j in 0..4096 {
                    core::ptr::write_volatile(ptr.add(j), 0);
                }

                curr += 4096;
            }

            if ph.filesz > 0 {
                let dest = start_addr as *mut u8;
                let src_offset = ph.offset as usize;
                let src_len = ph.filesz as usize;

                for j in 0..src_len {
                    let byte = data[src_offset + j];
                    core::ptr::write_volatile(dest.add(j), byte);
                }
            }
        }
    }

    Ok(header.entry)
}
