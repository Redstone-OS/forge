//! # Shebang Script Support
//!
//! Detecção e parsing de scripts com shebang (#!).
//!
//! # Status: STUB
//!
//! Este módulo atualmente apenas detecta shebangs mas não executa scripts.
//! A execução de scripts requer spawn recursivo com o interpretador.
//!
//! # TODO
//!
//! - [ ] Parsear linha do shebang para extrair interpretador e argumentos
//! - [ ] Spawn recursivo: `spawn(interpreter, [script_path, args...])`
//! - [ ] Limite de recursão para evitar loops infinitos
//! - [ ] Suporte a `/system/services/shell/` para interpretadores via PATH

/// Detecta se o arquivo é um script e retorna a linha do shebang
///
/// # Argumentos
///
/// * `data` - Conteúdo do arquivo
///
/// # Retorna
///
/// * `Some(interpreter_line)` - Linha após "#!" (sem newline)
/// * `None` - Se não for um script
///
/// # Exemplo
///
/// ```rust,ignore
/// let data = b"#!/bin/sh\necho hello";
/// assert_eq!(detect_shebang(data), Some("/bin/sh"));
/// ```
pub fn detect_shebang(data: &[u8]) -> Option<&str> {
    // Verificar magic "#!"
    if data.len() < 2 || &data[0..2] != b"#!" {
        return None;
    }

    // Encontrar fim da linha
    let mut end = 2;
    while end < data.len() && data[end] != b'\n' && data[end] != b'\r' {
        end += 1;
    }

    // Converter para string (ignorando erros UTF-8)
    core::str::from_utf8(&data[2..end]).ok().map(|s| s.trim())
}

/// Parseia a linha do shebang em interpretador e argumentos
///
/// # TODO: Implementar
///
/// ```rust,ignore
/// let (interp, args) = parse_shebang_line("/system/services/shell/bash")?;
/// // interp = "/system/services/shell/bash"
/// // args = []
/// ```
#[allow(dead_code)]
pub fn parse_shebang_line(_line: &str) -> Option<(&str, &str)> {
    // TODO: Implementar parsing de argumentos
    // Por enquanto retorna None para indicar não suportado
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shebang() {
        assert_eq!(detect_shebang(b"#!/bin/sh\n"), Some("/bin/sh"));
        assert_eq!(
            detect_shebang(b"#!/system/services/shell/bash\n"),
            Some("/system/services/shell/bash")
        );
        assert_eq!(detect_shebang(b"not a script"), None);
        assert_eq!(detect_shebang(b""), None);
        assert_eq!(detect_shebang(b"#"), None);
    }
}
