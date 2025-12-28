//! Parsing de caminhos

/// Iterador sobre componentes de caminho
pub struct PathComponents<'a> {
    remaining: &'a str,
}

impl<'a> PathComponents<'a> {
    pub fn new(path: &'a str) -> Self {
        // Remover / inicial
        let path = path.strip_prefix('/').unwrap_or(path);
        Self { remaining: path }
    }
}

impl<'a> Iterator for PathComponents<'a> {
    type Item = &'a str;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        
        match self.remaining.find('/') {
            Some(pos) => {
                let component = &self.remaining[..pos];
                self.remaining = &self.remaining[pos + 1..];
                Some(component)
            }
            None => {
                let component = self.remaining;
                self.remaining = "";
                Some(component)
            }
        }
    }
}

/// Verifica se caminho é absoluto
pub fn is_absolute(path: &str) -> bool {
    path.starts_with('/')
}

/// Normaliza caminho (remove . e ..)
pub fn normalize(path: &str) -> alloc::string::String {
    use alloc::vec::Vec;
    use alloc::string::String;
    
    let mut components: Vec<&str> = Vec::new();
    
    for comp in PathComponents::new(path) {
        match comp {
            "" | "." => continue,
            ".." => { components.pop(); }
            _ => components.push(comp),
        }
    }
    
    let mut result = String::from("/");
    for (i, comp) in components.iter().enumerate() {
        if i > 0 {
            result.push('/');
        }
        result.push_str(comp);
    }
    
    result
}
