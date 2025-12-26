//! Implementação ASM das operações de memória

/// Zera N bytes a partir de ptr usando ASM puro
#[inline]
pub unsafe fn memzero_asm(ptr: *mut u8, len: usize) {
    if len == 0 {
        return;
    }

    let qwords = len / 8;
    let remainder = len % 8;

    // Usa rep stosq para blocos de 8 bytes
    if qwords > 0 {
        let mut dst = ptr as *mut u64;
        core::arch::asm!(
            "rep stosq",
            inout("rcx") qwords => _,
            inout("rdi") dst => dst,
            in("rax") 0u64,
            options(nostack)
        );
    }

    // Bytes restantes um a um
    if remainder > 0 {
        let mut i = len - remainder;
        while i < len {
            core::arch::asm!(
                "mov byte ptr [{0}], 0",
                in(reg) ptr.add(i),
                options(nostack, preserves_flags)
            );
            i += 1;
        }
    }
}

/// Copia N bytes de src para dst usando ASM puro
#[inline]
pub unsafe fn memcpy_asm(dst: *mut u8, src: *const u8, len: usize) {
    if len == 0 {
        return;
    }

    let qwords = len / 8;
    let remainder = len % 8;

    // Copia blocos de 8 bytes
    if qwords > 0 {
        let mut d = dst as *mut u64;
        let mut s = src as *const u64;
        core::arch::asm!(
            "rep movsq",
            inout("rcx") qwords => _,
            inout("rdi") d => d,
            inout("rsi") s => s,
            options(nostack)
        );
    }

    // Bytes restantes
    if remainder > 0 {
        let base = len - remainder;
        let mut i = 0;
        while i < remainder {
            let byte: u8;
            core::arch::asm!(
                "mov {0}, [{1}]",
                out(reg_byte) byte,
                in(reg) src.add(base + i),
                options(nostack, preserves_flags, readonly)
            );
            core::arch::asm!(
                "mov [{0}], {1}",
                in(reg) dst.add(base + i),
                in(reg_byte) byte,
                options(nostack, preserves_flags)
            );
            i += 1;
        }
    }
}

/// Preenche N bytes com valor usando ASM puro
#[inline]
pub unsafe fn memset_asm(ptr: *mut u8, val: u8, len: usize) {
    if len == 0 {
        return;
    }

    // Expande byte para qword
    let qword_val: u64 = {
        let v = val as u64;
        v | (v << 8) | (v << 16) | (v << 24) | (v << 32) | (v << 40) | (v << 48) | (v << 56)
    };

    let qwords = len / 8;
    let remainder = len % 8;

    if qwords > 0 {
        let mut dst = ptr as *mut u64;
        core::arch::asm!(
            "rep stosq",
            inout("rcx") qwords => _,
            inout("rdi") dst => dst,
            in("rax") qword_val,
            options(nostack)
        );
    }

    if remainder > 0 {
        let mut i = len - remainder;
        while i < len {
            core::arch::asm!(
                "mov byte ptr [{0}], {1}",
                in(reg) ptr.add(i),
                in(reg_byte) val,
                options(nostack, preserves_flags)
            );
            i += 1;
        }
    }
}

/// Leitura volatile de u64 usando ASM
#[inline]
pub unsafe fn read_u64_asm(ptr: *const u64) -> u64 {
    let val: u64;
    core::arch::asm!(
        "mov {0}, [{1}]",
        out(reg) val,
        in(reg) ptr,
        options(nostack, preserves_flags, readonly)
    );
    val
}

/// Escrita volatile de u64 usando ASM
#[inline]
pub unsafe fn write_u64_asm(ptr: *mut u64, val: u64) {
    core::arch::asm!(
        "mov [{0}], {1}",
        in(reg) ptr,
        in(reg) val,
        options(nostack, preserves_flags)
    );
}

/// Leitura volatile de u8 usando ASM
#[inline]
pub unsafe fn read_u8_asm(ptr: *const u8) -> u8 {
    let val: u8;
    core::arch::asm!(
        "mov {0}, [{1}]",
        out(reg_byte) val,
        in(reg) ptr,
        options(nostack, preserves_flags, readonly)
    );
    val
}

/// Escrita volatile de u8 usando ASM
#[inline]
pub unsafe fn write_u8_asm(ptr: *mut u8, val: u8) {
    core::arch::asm!(
        "mov [{0}], {1}",
        in(reg) ptr,
        in(reg_byte) val,
        options(nostack, preserves_flags)
    );
}
