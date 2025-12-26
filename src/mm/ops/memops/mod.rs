//! # Operações de Memória
//!
//! Implementações duais (ASM e Rust) para operações de memória.
//! Usa feature flag `memops_asm` para escolher implementação.

// =============================================================================
// MODULOS DE IMPLEMENTAÇÃO
// =============================================================================

#[cfg(feature = "memops_asm")]
mod asm_impl;

#[cfg(not(feature = "memops_asm"))]
mod rust_impl;

// =============================================================================
// INTERFACE PÚBLICA
// =============================================================================

/// Zera N bytes a partir de ptr
///
/// # Safety
/// - `ptr` deve ser válido e apontar para memória acessível
/// - `len` bytes a partir de `ptr` devem ser acessíveis
#[inline(always)]
pub unsafe fn memzero(ptr: *mut u8, len: usize) {
    #[cfg(feature = "memops_asm")]
    asm_impl::memzero_asm(ptr, len);

    #[cfg(not(feature = "memops_asm"))]
    rust_impl::memzero_rust(ptr, len);
}

/// Copia N bytes de src para dst
///
/// # Safety
/// - `dst` e `src` devem ser válidos
/// - Regiões não devem se sobrepor (use memmove para overlap)
#[inline(always)]
pub unsafe fn memcpy(dst: *mut u8, src: *const u8, len: usize) {
    #[cfg(feature = "memops_asm")]
    asm_impl::memcpy_asm(dst, src, len);

    #[cfg(not(feature = "memops_asm"))]
    rust_impl::memcpy_rust(dst, src, len);
}

/// Preenche N bytes com valor
///
/// # Safety
/// - `ptr` deve ser válido e apontar para memória acessível
#[inline(always)]
pub unsafe fn memset(ptr: *mut u8, val: u8, len: usize) {
    #[cfg(feature = "memops_asm")]
    asm_impl::memset_asm(ptr, val, len);

    #[cfg(not(feature = "memops_asm"))]
    rust_impl::memset_rust(ptr, val, len);
}

/// Leitura volatile de u64
#[inline(always)]
pub unsafe fn read_u64(ptr: *const u64) -> u64 {
    #[cfg(feature = "memops_asm")]
    return asm_impl::read_u64_asm(ptr);

    #[cfg(not(feature = "memops_asm"))]
    return rust_impl::read_u64_rust(ptr);
}

/// Escrita volatile de u64
#[inline(always)]
pub unsafe fn write_u64(ptr: *mut u64, val: u64) {
    #[cfg(feature = "memops_asm")]
    asm_impl::write_u64_asm(ptr, val);

    #[cfg(not(feature = "memops_asm"))]
    rust_impl::write_u64_rust(ptr, val);
}

/// Leitura volatile de u8
#[inline(always)]
pub unsafe fn read_u8(ptr: *const u8) -> u8 {
    #[cfg(feature = "memops_asm")]
    return asm_impl::read_u8_asm(ptr);

    #[cfg(not(feature = "memops_asm"))]
    return rust_impl::read_u8_rust(ptr);
}

/// Escrita volatile de u8
#[inline(always)]
pub unsafe fn write_u8(ptr: *mut u8, val: u8) {
    #[cfg(feature = "memops_asm")]
    asm_impl::write_u8_asm(ptr, val);

    #[cfg(not(feature = "memops_asm"))]
    rust_impl::write_u8_rust(ptr, val);
}

// =============================================================================
// TESTES
// =============================================================================

// TODO: Fix tests
// #[cfg(test)]
// mod tests { ... }
