//! # Funções de Memória
//!
//! Implementações de funções de memória estilo C.
//! Usadas no early boot antes do allocator estar disponível.
//!
//! ## Safety
//!
//! Todas as funções neste módulo são `unsafe` pois operam
//! diretamente com ponteiros brutos.

/// Preenche memória com um byte.
///
/// # Safety
///
/// - `dest` deve ser válido para `count` bytes de escrita
/// - Não pode ter overlap com outras escritas concorrentes
#[inline]
pub unsafe fn memset(dest: *mut u8, value: u8, count: usize) {
    let mut ptr = dest;
    let mut remaining = count;

    // Otimização: preencher palavras de 8 bytes quando possível
    let value64 = (value as u64) * 0x0101_0101_0101_0101;

    // Alinha ao limite de 8 bytes
    while remaining > 0 && (ptr as usize) % 8 != 0 {
        *ptr = value;
        ptr = ptr.add(1);
        remaining -= 1;
    }

    // Preenche palavras de 8 bytes
    let ptr64 = ptr as *mut u64;
    let words = remaining / 8;
    for i in 0..words {
        *ptr64.add(i) = value64;
    }

    // Preenche bytes restantes
    ptr = ptr.add(words * 8);
    remaining %= 8;
    while remaining > 0 {
        *ptr = value;
        ptr = ptr.add(1);
        remaining -= 1;
    }
}

/// Copia memória (regiões NÃO podem sobrepor).
///
/// # Safety
///
/// - `dest` e `src` devem ser válidos para `count` bytes
/// - As regiões NÃO podem ter overlap (use memmove se houver)
#[inline]
pub unsafe fn memcpy(dest: *mut u8, src: *const u8, count: usize) {
    let mut d = dest;
    let mut s = src;
    let mut remaining = count;

    // Otimização: copiar palavras de 8 bytes quando alinhado
    if (d as usize) % 8 == (s as usize) % 8 {
        // Alinha
        while remaining > 0 && (d as usize) % 8 != 0 {
            *d = *s;
            d = d.add(1);
            s = s.add(1);
            remaining -= 1;
        }

        // Copia palavras
        let d64 = d as *mut u64;
        let s64 = s as *const u64;
        let words = remaining / 8;
        for i in 0..words {
            *d64.add(i) = *s64.add(i);
        }

        d = d.add(words * 8);
        s = s.add(words * 8);
        remaining %= 8;
    }

    // Copia bytes restantes
    while remaining > 0 {
        *d = *s;
        d = d.add(1);
        s = s.add(1);
        remaining -= 1;
    }
}

/// Copia memória (regiões PODEM sobrepor).
///
/// # Safety
///
/// - `dest` e `src` devem ser válidos para `count` bytes
#[inline]
pub unsafe fn memmove(dest: *mut u8, src: *const u8, count: usize) {
    if (dest as usize) < (src as usize) || (dest as usize) >= (src as usize) + count {
        // Sem overlap ou dest antes de src - copia para frente
        memcpy(dest, src, count);
    } else {
        // dest dentro de src - copia de trás para frente
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

/// Compara memória.
///
/// # Retorno
/// - `0` se iguais
/// - `< 0` se `a < b`
/// - `> 0` se `a > b`
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

/// Procura um byte em memória.
///
/// # Retorno
/// - Ponteiro para primeira ocorrência
/// - `null` se não encontrado
///
/// # Safety
///
/// - `ptr` deve ser válido para `count` bytes
#[inline]
pub unsafe fn memchr(ptr: *const u8, value: u8, count: usize) -> *const u8 {
    let mut p = ptr;
    for _ in 0..count {
        if *p == value {
            return p;
        }
        p = p.add(1);
    }
    core::ptr::null()
}

/// Zera memória de forma segura (não otimizada pelo compilador).
///
/// Útil para limpar dados sensíveis (chaves, senhas).
///
/// # Safety
///
/// - `dest` deve ser válido para `count` bytes
#[inline(never)]
pub unsafe fn memzero_explicit(dest: *mut u8, count: usize) {
    core::ptr::write_bytes(dest, 0, count);
    // Fence para garantir que o compilador não otimize
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}
