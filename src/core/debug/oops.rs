//! # Kernel Oops
//!
//! Tratamento de erros recuperáveis.
//!
//! Diferente de um Panic (que para tudo), um Oops sinaliza um bug
//! que afetou apenas parte do sistema.

/// Sinaliza erro grave mas possivelmente recuperável.
///
/// O kernel tentará continuar executando após um oops.
#[cold]
pub fn oops(msg: &str) {
    crate::kerror!("*****************************************************");
    crate::kerror!("*                   KERNEL OOPS                     *");
    crate::kerror!("*****************************************************");
    crate::kerror!("Message:", msg);

    // TODO: Dump stack trace
    // TODO: Kill current process if in process context

    crate::kerror!("*****************************************************");
    crate::kerror!("* TENTANDO RECUPERAR...                             *");
    crate::kerror!("*****************************************************");
}

/// Oops com informação adicional.
#[cold]
pub fn oops_at(msg: &str, file: &str, line: u32) {
    crate::kerror!("*****************************************************");
    crate::kerror!("*                   KERNEL OOPS                     *");
    crate::kerror!("*****************************************************");
    crate::kerror!("Message:", msg);
    crate::kerror!("File:", file);
    crate::kerror!("Line:", line);
    crate::kerror!("*****************************************************");
}

/// Macro para oops com localização automática.
#[macro_export]
macro_rules! koops {
    ($msg:expr) => {
        $crate::core::debug::oops::oops_at($msg, file!(), line!())
    };
}
