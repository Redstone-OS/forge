//! # C-String Functions
//!
//! Funções para manipulação de strings estilo C (terminadas em null).
//! Útil para interoperabilidade com hardware/firmware.

/// Calcula tamanho de string terminada em null.
///
/// ## Safety
///
/// - `s` deve apontar para uma string válida terminada em null
/// - A string não pode ter mais de `isize::MAX` bytes
#[inline]
pub unsafe fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

/// Compara duas strings C.
///
/// ## Retorno:
/// - `0` se iguais
/// - `< 0` se s1 < s2
/// - `> 0` se s1 > s2
///
/// ## Safety
///
/// - Ambas strings devem ser válidas e terminadas em null
#[inline]
pub unsafe fn strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut i = 0;
    loop {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);

        if c1 != c2 || c1 == 0 {
            return (c1 as i32) - (c2 as i32);
        }
        i += 1;
    }
}

/// Compara duas strings C com limite de bytes.
///
/// ## Safety
///
/// - Ambas strings devem ter pelo menos `n` bytes ou ser terminadas antes
#[inline]
pub unsafe fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);

        if c1 != c2 {
            return (c1 as i32) - (c2 as i32);
        }
        if c1 == 0 {
            return 0;
        }
    }
    0
}

/// Copia string C.
///
/// ## Safety
///
/// - `dest` deve ter espaço suficiente
/// - `src` deve ser terminada em null
/// - As regiões não podem sobrepor
#[inline]
pub unsafe fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8 {
    let mut i = 0;
    loop {
        let c = *src.add(i);
        *dest.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    dest
}

/// Copia string C com limite.
///
/// ## Safety
///
/// - `dest` deve ter pelo menos `n` bytes
/// - Se `src` for menor que `n`, preenche com zeros
#[inline]
pub unsafe fn strncpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;

    // Copia a string
    while i < n {
        let c = *src.add(i);
        *dest.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }

    // Preenche restante com zeros
    while i < n {
        *dest.add(i) = 0;
        i += 1;
    }

    dest
}

/// Encontra caractere em string C.
///
/// ## Retorno:
/// - Ponteiro para primeira ocorrência
/// - Null se não encontrado
///
/// ## Safety
///
/// - `s` deve ser terminada em null
#[inline]
pub unsafe fn strchr(s: *const u8, c: u8) -> *const u8 {
    let mut ptr = s;
    loop {
        let ch = *ptr;
        if ch == c {
            return ptr;
        }
        if ch == 0 {
            return core::ptr::null();
        }
        ptr = ptr.add(1);
    }
}

/// Tokenizer seguro para strings Rust.
///
/// Divide uma string em tokens baseado em um delimitador.
pub struct Tokenizer<'a> {
    rest: &'a str,
    delim: char,
}

impl<'a> Tokenizer<'a> {
    /// Cria um novo tokenizer.
    pub fn new(s: &'a str, delim: char) -> Self {
        Self { rest: s, delim }
    }

    /// Retorna o restante da string não processada.
    pub fn rest(&self) -> &'a str {
        self.rest
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }

        match self.rest.find(self.delim) {
            Some(idx) => {
                let token = &self.rest[..idx];
                self.rest = &self.rest[idx + 1..];
                Some(token)
            }
            None => {
                let token = self.rest;
                self.rest = "";
                Some(token)
            }
        }
    }
}

/// Divide string em partes usando delimitador.
///
/// Versão mais simples do Tokenizer para uso rápido.
pub fn split<'a>(s: &'a str, delim: char) -> Tokenizer<'a> {
    Tokenizer::new(s, delim)
}

/// Verifica se string começa com prefixo.
#[inline]
pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

/// Verifica se string termina com sufixo.
#[inline]
pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

/// Remove espaços em branco do início e fim.
#[inline]
pub fn trim(s: &str) -> &str {
    s.trim()
}
