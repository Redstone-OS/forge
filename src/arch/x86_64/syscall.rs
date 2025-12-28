//! Configuração de Syscall (instrução syscall/sysret)

/// Habilita syscalls via MSRs (EFER, STAR, LSTAR, SFMASK)
///
/// # Safety
///
/// Escreve em MSRs críticos.
pub unsafe fn enable_syscalls() {
    // TODO: Configurar EFER.SCE (System Call Enable)
    // TODO: Configurar STAR (Selectores CS/SS)
    // TODO: Configurar LSTAR (Endereço do handler)
    // TODO: Configurar SFMASK (Flags para limpar)
}
