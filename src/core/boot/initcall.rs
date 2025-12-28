//! Arquivo: core/boot/initcall.rs
//!
//! Propósito: Sistema de Initcalls de inicialização (Initcalls).
//! Permite que módulos e subsistemas registrem funções para serem executadas
//! automaticamente durante o boot, sem precisar poluir o `kmain` com chamadas manuais.
//!
//! Detalhes de Implementação:
//! - Baseado em seções do Linker (`.init_array`).
//! - As funções são coletadas em um array de ponteiros de função.
//! - O kernel itera e executa cada uma em ordem.
//!
//! Initcalls

pub type InitCall = fn() -> Result<(), &'static str>;

// Símbolos definidos pelo Linker Script
extern "C" {
    static __init_array_start: u8;
    static __init_array_end: u8;
}

/// Executa todas as initcalls registradas.
/// Deve ser chamado proximo ao final da inicialização do kernel, antes do userspace.
pub fn run_initcalls() {
    crate::kinfo!("Executando Initcalls...");

    let start = unsafe { &raw const __init_array_start as *const InitCall };
    let end = unsafe { &raw const __init_array_end as *const InitCall };

    let count = (end as usize - start as usize) / core::mem::size_of::<InitCall>();

    crate::kinfo!("Total de initcalls: ", count as u64);

    // Iterar pointer arithmetic
    for i in 0..count {
        unsafe {
            let func_ptr = start.add(i).read();
            match func_ptr() {
                Ok(_) => {}
                Err(e) => {
                    crate::kerror!("Initcall falhou: ");
                    crate::kerror!(e);
                    // Dependendo da política, poderíamos pânico ou apenas logar
                }
            }
        }
    }
}

/// Macro para registrar uma função como initcall.
#[macro_export]
macro_rules! define_initcall {
    ($func:path) => {
        #[link_section = ".init_array"]
        #[used] // Impede que o compilador remova se não for usado explicitamente
        #[allow(non_upper_case_globals)]
        static __INITCALL_PTR: $crate::core::boot::initcall::InitCall = $func;
    };
}
