//! Shebang Script Support (#!)

/// Verifica se o arquivo é um script e retorna o interpretador
pub fn check_shebang(data: &[u8]) -> Option<&str> {
    if data.len() < 2 || &data[0..2] != b"#!" {
        return None;
    }
    
    // Encontrar fim da linha
    let mut end = 2;
    while end < data.len() && data[end] != b'\n' {
        end += 1;
    }
    
    // Converter para string (ignorando erros UTF-8 para simplificar neste nível)
    core::str::from_utf8(&data[2..end]).ok().map(|s| s.trim())
}
