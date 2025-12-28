/// Arquivo: klib/string/string.rs
///
/// Propósito: Manipulação de strings de baixo nível (estilo C).
/// Útil para lidar com buffers crus, command line, e interação com hardware/firmware.
///
/// Detalhes de Implementação:
/// - Operações byte-a-byte.

// String Utils

/// Calcula tamanho de string terminada em nulo
pub fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

/// Compara duas strings (estilo strcmp)
pub fn strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut i = 0;
    unsafe {
        while *s1.add(i) != 0 && *s2.add(i) != 0 {
            if *s1.add(i) != *s2.add(i) {
                return (*s1.add(i) as i32) - (*s2.add(i) as i32);
            }
            i += 1;
        }
        (*s1.add(i) as i32) - (*s2.add(i) as i32)
    }
}

/// Compara duas strings com limite
pub fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;
    unsafe {
        while i < n && *s1.add(i) != 0 && *s2.add(i) != 0 {
            if *s1.add(i) != *s2.add(i) {
                return (*s1.add(i) as i32) - (*s2.add(i) as i32);
            }
            i += 1;
        }
        if i == n {
            0
        } else {
            (*s1.add(i) as i32) - (*s2.add(i) as i32)
        }
    }
}

/// Tokenizer simples (strtok-like)
/// Nota: Altera a string original substituindo separador por \0 se for mutável.
/// Aqui implementamos versão segura que retorna fatias (slices) de &str.
pub struct Tokenizer<'a> {
    rest: &'a str,
    delim: char,
}

impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a str, delim: char) -> Self {
        Self { rest: s, delim }
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
