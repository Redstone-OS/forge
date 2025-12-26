use core::alloc::Layout;

/// Handler global de erro de alocação (OOM)
/// ----------------------------------------
/// Chamado pelo Rust (`alloc` crate) quando uma alocação falha (retorna null)
/// e o caller não tratou o erro (ex: `Box::new` falhando).
///
/// Atualmente: Panic Kernel Panic.
/// Futuro: Tentar recuperar, matar processos, ou hibernar.
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    crate::kerror!("(OOM) CRITICAL: Kernel Out of Memory!");
    crate::kerror!("(OOM) Falha ao alocar layout: {:?}", layout);

    // TODO: Adicionar dump de estatísticas de memória aqui

    panic!("Kernel OOM");
}
