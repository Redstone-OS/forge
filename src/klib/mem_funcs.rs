//! Funções de memória (sem SSE)

/// Preenche memória com byte
/// 
/// # Safety
/// 
/// - `dest` deve ser válido para `count` bytes
/// - Não pode ter overlap com outras escritas
#[inline]
pub unsafe fn memset(dest: *mut u8, value: u8, count: usize) {
    let mut ptr = dest;
    let mut remaining = count;
    
    while remaining > 0 {
        *ptr = value;
        ptr = ptr.add(1);
        remaining -= 1;
    }
}

/// Copia memória (não pode ter overlap)
/// 
/// # Safety
/// 
/// - `dest` e `src` devem ser válidos para `count` bytes
/// - As regiões não podem ter overlap
#[inline]
pub unsafe fn memcpy(dest: *mut u8, src: *const u8, count: usize) {
    let mut d = dest;
    let mut s = src;
    let mut remaining = count;
    
    while remaining > 0 {
        *d = *s;
        d = d.add(1);
        s = s.add(1);
        remaining -= 1;
    }
}

/// Copia memória (pode ter overlap)
/// 
/// # Safety
/// 
/// - `dest` e `src` devem ser válidos para `count` bytes
#[inline]
pub unsafe fn memmove(dest: *mut u8, src: *const u8, count: usize) {
    if (dest as usize) < (src as usize) {
        // Copia para frente
        memcpy(dest, src, count);
    } else {
        // Copia de trás para frente
        let mut d = dest.add(count);
        let mut s = src.add(count);
        let mut remaining = count;
        
        while remaining > 0 {
            d = d.sub(1);
            s = s.sub(1);
            *d = *s;
            remaining -= 1;
        }
    }
}

/// Compara memória
/// 
/// # Safety
/// 
/// - `a` e `b` devem ser válidos para `count` bytes
#[inline]
pub unsafe fn memcmp(a: *const u8, b: *const u8, count: usize) -> i32 {
    let mut pa = a;
    let mut pb = b;
    
    for _ in 0..count {
        let va = *pa;
        let vb = *pb;
        if va != vb {
            return (va as i32) - (vb as i32);
        }
        pa = pa.add(1);
        pb = pb.add(1);
    }
    0
}
