//! Handlers de Interrupção

/// Handler para exceções de CPU
pub extern "x86-interrupt" fn exception_handler() {
    // TODO: Implementar handler genérico
}

/// Handler para Page Fault (#PF)
pub extern "x86-interrupt" fn page_fault_handler() {
    // TODO: Ler CR2 e tratar fault
}

/// Handler para General Protection Fault (#GP)
pub extern "x86-interrupt" fn double_fault_handler() {
    // TODO: Panic!
}
