/// Arquivo: core/debug/oops.rs
///
/// Propósito: Tratamento de erros recuperáveis do kernel ("Oops").
/// Diferente de um Panic (que para tudo), um Oops sinaliza um bug ou estado inválido
/// que afetou apenas o processo atual ou uma parte não-portante do sistema.
///
/// Detalhes de Implementação:
/// - Loga o erro de forma visível.
/// - (Futuro) Matará o processo atual se estiver em contexto de processo.
/// - (Futuro) Tenta liberar locks segurados para evitar deadlocks (best effort).

//! Kernel Oops (Erros recuperáveis)

/// Sinaliza um erro grave mas possivelmente recuperável.
///
/// Use isso quando algo inesperado acontece (ex: bug em driver) mas o kernel
/// acredita que pode continuar rodando (matando o processo ofensor, por exemplo).
pub fn oops(msg: &str) {
    crate::kerror!("*****************************************************");
    crate::kerror!("*                   KERNEL OOPS                     *");
    crate::kerror!("*****************************************************");
    crate::kerror!("Message:", 0); // TODO: suporte a str
    crate::kerror!(msg);
    
    // TODO: Dump stack trace
    // TODO: Se (current_process != NULL) kill(current_process)
    // TODO: Decrementar refcounts se necessário
    
    crate::kerror!("*****************************************************");
    crate::kerror!("* TENTANDO RECUPERAR... SE O SISTEMA TRAVAR, REINICIE *");
    crate::kerror!("*****************************************************");
}
