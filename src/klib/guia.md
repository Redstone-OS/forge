# Guia de Implementação: `klib/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE.

---

## 1. PROPÓSITO

Biblioteca de utilitários do kernel. Substitui partes da `std` que não existem em `no_std`.

---

## 2. ESTRUTURA

```
klib/
├── mod.rs              ✅ JÁ EXISTE
├── align.rs            → Alinhamento de memória
├── bitmap.rs           → Bitmap genérico
├── mem_funcs.rs        → memset, memcpy, memmove
├── test_framework.rs   → Framework de testes
├── hash/
│   ├── mod.rs
│   └── hashtable.rs    → Tabela hash
├── list/
│   ├── mod.rs
│   └── linked.rs       → Lista duplamente ligada
├── string/
│   ├── mod.rs
│   └── string.rs       → Manipulação de strings
└── tree/
    ├── mod.rs
    └── rbtree.rs       → Red-Black Tree
```

---

## 3. REGRAS

### ❌ NUNCA:
- Usar `std::`
- Usar `f32`/`f64`
- Alocar heap nas funções básicas (align, mem_funcs)
- Usar `unwrap()`

### ✅ SEMPRE:
- Usar `core::` ao invés de `std::`
- Funções `const fn` quando possível
- Inline em funções pequenas

---

## 4. IMPLEMENTAÇÕES

### 4.1 `align.rs`

```rust
//! Funções de alinhamento de memória

/// Alinha valor para cima
#[inline]
pub const fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + align - 1) & !(align - 1)
}

/// Alinha valor para baixo
#[inline]
pub const fn align_down(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    value & !(align - 1)
}

/// Verifica se está alinhado
#[inline]
pub const fn is_aligned(value: usize, align: usize) -> bool {
    debug_assert!(align.is_power_of_two());
    (value & (align - 1)) == 0
}
```

### 4.2 `bitmap.rs`

```rust
//! Bitmap genérico

/// Bitmap para gerenciamento de bits
pub struct Bitmap<'a> {
    data: &'a mut [u64],
    len: usize,
}

impl<'a> Bitmap<'a> {
    /// Cria bitmap sobre slice existente
    pub fn new(data: &'a mut [u64], bits: usize) -> Self {
        Self { data, len: bits }
    }
    
    /// Define um bit
    pub fn set(&mut self, index: usize) {
        debug_assert!(index < self.len);
        let word = index / 64;
        let bit = index % 64;
        self.data[word] |= 1 << bit;
    }
    
    /// Limpa um bit
    pub fn clear(&mut self, index: usize) {
        debug_assert!(index < self.len);
        let word = index / 64;
        let bit = index % 64;
        self.data[word] &= !(1 << bit);
    }
    
    /// Testa um bit
    pub fn test(&self, index: usize) -> bool {
        debug_assert!(index < self.len);
        let word = index / 64;
        let bit = index % 64;
        (self.data[word] & (1 << bit)) != 0
    }
    
    /// Encontra primeiro bit livre (0)
    pub fn find_first_zero(&self) -> Option<usize> {
        for (i, &word) in self.data.iter().enumerate() {
            if word != u64::MAX {
                let bit = word.trailing_ones() as usize;
                let index = i * 64 + bit;
                if index < self.len {
                    return Some(index);
                }
            }
        }
        None
    }
}
```

### 4.3 `mem_funcs.rs`

```rust
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
```

### 4.4 `test_framework.rs`

```rust
//! Framework de testes do kernel

/// Resultado de teste
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestResult {
    Passed,
    Failed,
    Skipped,
}

/// Um caso de teste
pub struct TestCase {
    pub name: &'static str,
    pub func: fn() -> TestResult,
}

/// Executa suite de testes
pub fn run_test_suite(name: &str, tests: &[TestCase]) -> (usize, usize, usize) {
    crate::kinfo!("=== Executando suite:", name.as_ptr() as u64);
    
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    
    for test in tests {
        let result = (test.func)();
        match result {
            TestResult::Passed => {
                crate::kinfo!("[PASS]", test.name.as_ptr() as u64);
                passed += 1;
            }
            TestResult::Failed => {
                crate::kerror!("[FAIL]", test.name.as_ptr() as u64);
                failed += 1;
            }
            TestResult::Skipped => {
                crate::kwarn!("[SKIP]", test.name.as_ptr() as u64);
                skipped += 1;
            }
        }
    }
    
    crate::kinfo!("Resultados: passed=", passed as u64);
    (passed, failed, skipped)
}
```

---

## 5. DEPENDÊNCIAS

Este módulo NÃO pode importar de nenhum outro módulo exceto `core`.

---

## 6. CHECKLIST

- [ ] align.rs usa apenas operações bitwise
- [ ] mem_funcs.rs não usa SSE (loop byte a byte)
- [ ] bitmap.rs funciona com qualquer tamanho
- [ ] Todas as funções são `const fn` ou `#[inline]` quando aplicável
