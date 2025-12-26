//! Implementação Rust (safe/debug) das operações de memória
//!
//! Usado quando a feature `memops_asm` está desabilitada ou para validação cruzada.

/// Zera N bytes a partir de ptr usando Rust volátil
#[inline]
pub unsafe fn memzero_rust(ptr: *mut u8, len: usize) {
    for i in 0..len {
        ptr.add(i).write_volatile(0);
    }
}

/// Copia N bytes de src para dst usando Rust volátil
#[inline]
pub unsafe fn memcpy_rust(dst: *mut u8, src: *const u8, len: usize) {
    for i in 0..len {
        // Leitura e escrita volátil para evitar otimizações que poderiam
        // transformar isso de volta em memcpy intrínseco se fossem ponteiros normais
        let val = src.add(i).read_volatile();
        dst.add(i).write_volatile(val);
    }
}

/// Preenche N bytes com valor usando Rust volátil
#[inline]
pub unsafe fn memset_rust(ptr: *mut u8, val: u8, len: usize) {
    for i in 0..len {
        ptr.add(i).write_volatile(val);
    }
}

/// Leitura volatile de u64
#[inline]
pub unsafe fn read_u64_rust(ptr: *const u64) -> u64 {
    ptr.read_volatile()
}

/// Escrita volatile de u64
#[inline]
pub unsafe fn write_u64_rust(ptr: *mut u64, val: u64) {
    ptr.write_volatile(val);
}

/// Leitura volatile de u8
#[inline]
pub unsafe fn read_u8_rust(ptr: *const u8) -> u8 {
    ptr.read_volatile()
}

/// Escrita volatile de u8
#[inline]
pub unsafe fn write_u8_rust(ptr: *mut u8, val: u8) {
    ptr.write_volatile(val);
}
