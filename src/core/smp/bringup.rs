//! Bringup de APs (Application Processors)
//!
//! Responsável por acordar outros núcleos da CPU.

use super::topology::CpuId;
use crate::core::smp::ipi::{send_ipi, IpiTarget, IpiVector};

/// Inicializa o subsistema de SMP
pub fn init() {
    crate::kinfo!("(SMP) Init");
    // TODO: Detectar CPUs e iniciar APs via wake_ap
}

/// Tenta acordar uma CPU específica.
///
/// # Argumentos
///
/// * `apic_id`: ID de hardware da CPU alvo.
/// * `trampoline_addr`: Endereço físico onde o código de inicialização do AP está carregado.
///   Deve ser alinhado a 4KB (Page aligned) para o vetor SIPI (Vector = Addr >> 12).
///
/// # Retorno
///
/// Retorna `Ok(())` se a sequência foi enviada. Não garante que a CPU acordou (precisa verificar flag na memória).
pub unsafe fn wake_ap(apic_id: u32, trampoline_addr: u64) -> Result<(), &'static str> {
    crate::kinfo!("Tentando acordar AP com APIC ID: ", apic_id as u64);

    // Verificação de alinhamento do trampolim (SIPI vector é de 8 bits, endereçando páginas de 4k)
    if trampoline_addr & 0xFFF != 0 {
        return Err("Endereço do trampolim deve ser alinhado a 4KB");
    }

    if trampoline_addr > 0xFF000 {
        return Err("Endereço do trampolim deve estar na memória baixa (< 1MB) para modo real");
    }

    let sipi_vector = (trampoline_addr >> 12) as u8;

    // TODO: Usar funções de envio de IPI raw da arquitetura, pois send_ipi genérico usa vetor da IDT,
    // e aqui precisamos enviar sinais INIT/STARTUP especiais que não são vetores normais.
    // Como não expusemos isso ainda, deixaremos como TODO descritivo.

    // Sequência padrão x86 INIT-SIPI-SIPI:

    // 1. Enviar INIT IPI
    // crate::arch::apic::send_init_ipi(apic_id);
    crate::kdebug!("Enviando INIT...");

    // 2. Esperar 10ms
    // crate::arch::delay(10_000);

    // 3. Enviar SIPI (Startup IPI) com o vetor do trampolim
    // crate::arch::apic::send_sipi(apic_id, sipi_vector);
    crate::kdebug!("Enviando SIPI 1 (Vector ", sipi_vector as u64);

    // 4. Esperar 200us
    // crate::arch::delay(200);

    // 5. Enviar segundo SIPI (resiliência)
    // crate::arch::apic::send_sipi(apic_id, sipi_vector);
    crate::kdebug!("Enviando SIPI 2");

    Ok(())
}
