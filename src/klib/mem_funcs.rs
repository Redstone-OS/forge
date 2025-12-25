//! Implementações personalizadas de memset/memcpy/memmove.
//!
//! Estas funções substituem as versões do compiler_builtins que podem
//! gerar código inválido em bare-metal x86_64.
//!
//! O atributo `#[no_mangle]` garante que estas funções substituam
//! as implementações padrão do compilador.

/// Preenche `n` bytes de memória com o valor `c`.
///
/// # Safety
/// - `s` deve ser um ponteiro válido para uma região de pelo menos `n` bytes.
/// - A região deve ser gravável.
#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let byte = c as u8;
    let mut ptr = s;
    let end = s.add(n);

    // Zerar 8 bytes por vez quando possível (mais eficiente)
    if byte == 0 && n >= 8 {
        let aligned_end = s.add(n & !7);
        let ptr64 = ptr as *mut u64;
        let count = ((aligned_end as usize) - (ptr as usize)) / 8;

        for i in 0..count {
            core::ptr::write_volatile(ptr64.add(i), 0u64);
        }
        ptr = aligned_end;
    }

    // Bytes restantes (ou todos se não for zero)
    while ptr < end {
        core::ptr::write_volatile(ptr, byte);
        ptr = ptr.add(1);
    }

    s
}

/// Copia `n` bytes de `src` para `dest`.
///
/// # Safety
/// - `dest` e `src` devem ser ponteiros válidos para regiões de pelo menos `n` bytes.
/// - As regiões NÃO devem se sobrepor (use memmove para regiões sobrepostas).
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut d = dest;
    let mut s = src;
    let end = dest.add(n);

    // Copiar 8 bytes por vez quando alinhado
    if n >= 8 && ((dest as usize) & 7) == 0 && ((src as usize) & 7) == 0 {
        let aligned_end = dest.add(n & !7);
        let d64 = d as *mut u64;
        let s64 = s as *const u64;
        let count = ((aligned_end as usize) - (d as usize)) / 8;

        for i in 0..count {
            core::ptr::write_volatile(d64.add(i), core::ptr::read_volatile(s64.add(i)));
        }
        d = aligned_end;
        s = src.add(count * 8);
    }

    // Bytes restantes
    while d < end {
        core::ptr::write_volatile(d, core::ptr::read_volatile(s));
        d = d.add(1);
        s = s.add(1);
    }

    dest
}

/// Move `n` bytes de `src` para `dest`, lidando corretamente com sobreposição.
///
/// # Safety
/// - `dest` e `src` devem ser ponteiros válidos para regiões de pelo menos `n` bytes.
#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if src < dest as *const u8 && (src as usize + n) > dest as usize {
        // Cópia reversa para evitar sobrescrita
        let mut d = dest.add(n);
        let mut s = src.add(n);

        for _ in 0..n {
            d = d.sub(1);
            s = s.sub(1);
            core::ptr::write_volatile(d, core::ptr::read_volatile(s));
        }
    } else {
        // Cópia normal
        memcpy(dest, src, n);
    }

    dest
}

/// Compara `n` bytes de memória.
#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = core::ptr::read_volatile(s1.add(i));
        let b = core::ptr::read_volatile(s2.add(i));
        if a != b {
            return (a as i32) - (b as i32);
        }
    }
    0
}

/// Procura o byte `c` nos primeiros `n` bytes de `s`.
#[no_mangle]
pub unsafe extern "C" fn memchr(s: *const u8, c: i32, n: usize) -> *const u8 {
    let byte = c as u8;
    for i in 0..n {
        if core::ptr::read_volatile(s.add(i)) == byte {
            return s.add(i);
        }
    }
    core::ptr::null()
}

/// Calcula o comprimento de uma string C.
#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    while core::ptr::read_volatile(s.add(len)) != 0 {
        len += 1;
    }
    len
}
