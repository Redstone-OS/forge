//! # Compiler Builtins
//!
//! Fornece implementações básicas de memcpy, memset, etc. que o compilador
//! pode chamar automaticamente. Marcadas com #[no_mangle] para o linker.

use crate::mm::ops::memops;

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    memops::memcpy(dest, src, n);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    memops::memset(s, c as u8, n);
    s
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if (dest as usize) <= (src as usize) || (dest as usize) >= (src as usize) + n {
        memops::memcpy(dest, src, n);
    } else {
        // Cópia reversa para overlap
        for i in (0..n).rev() {
            dest.add(i).write_volatile(src.add(i).read_volatile());
        }
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = s1.add(i).read_volatile();
        let b = s2.add(i).read_volatile();
        if a != b {
            return a as i32 - b as i32;
        }
    }
    0
}
