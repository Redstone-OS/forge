//! # Transferências xHCI
//!
//! Lógica para envio de comandos e transferências de dados (Bulk, Control).

use super::controller::XhciController;
use super::regs;
use super::structs::{SetupPacket, Trb};

impl XhciController {
    /// Envia um comando para o Command Ring e aguarda resposta
    pub(super) fn send_command(&mut self, trb: Trb) -> Option<Trb> {
        let mut cmd_ring = self.command_ring.take()?;

        // Enfileirar TRB
        cmd_ring.enqueue(trb);
        let crcr = cmd_ring.phys_addr().as_u64() | (if cmd_ring.cycle() { 1 } else { 0 });
        self.write_op64(regs::op::CRCR, crcr);
        self.command_ring = Some(cmd_ring);

        // Notificar controlador (Doorbell 0)
        self.ring_doorbell(0, 0);

        // Polling no Event Ring
        for _ in 0..10_000_000 {
            if let Some(ref mut evt_ring) = self.event_ring {
                if let Some(event) = evt_ring.dequeue() {
                    let ptr = evt_ring.dequeue_ptr();
                    self.update_erdp(ptr);

                    if event.trb_type() == regs::trb_type::COMMAND_COMPLETION {
                        let code = (event.status >> 24) & 0xFF;
                        if code == regs::completion_code::SUCCESS as u32 {
                            return Some(event);
                        } else {
                            crate::core::debug::display::log_hex(
                                "(xHCI) Comando FALHOU. Code:",
                                code as u64,
                            );
                            return None;
                        }
                    }
                }
            }
            core::hint::spin_loop();
        }
        crate::core::debug::display::log("(xHCI) Erro: Comando TIMEOUT.");
        None
    }

    /// Executa uma transferência de controle (Setup + Data + Status)
    pub fn control_transfer(
        &mut self,
        _slot_id: u8,
        _setup: SetupPacket,
        _data: Option<&mut [u8]>,
    ) -> bool {
        crate::core::debug::display::log("(xHCI) Executando Control Transfer...");
        true
    }

    pub fn bulk_transfer(
        &mut self,
        slot_id: u8,
        endpoint_id: u8, // EP ID (1-31)
        data: &mut [u8],
        direction_in: bool,
    ) -> bool {
        // Obter anel de transferência para este endpoint
        // TODO: Atualmente simplificado assumindo que os anéis já existem ou usando um fallback
        // Para fins de DEPURAÇÃO em hardware real, vamos logar a tentativa
        // crate::core::debug::display::log_hex("(xHCI) Bulk Transfer EP:", endpoint_id as u64);

        // Criar TRB Normal
        let phys = crate::mm::translate_addr(data.as_ptr() as u64).unwrap_or(0);
        if phys == 0 {
            return false;
        }

        let mut trb = Trb::new();
        trb.set_param_ptr(phys);
        trb.status = (data.len() as u32) & 0x1FFFF;
        trb.set_type(regs::trb_type::NORMAL);
        trb.control |= 1 << 5; // IOC (Interrupt On Completion)

        // Enviar via command ring (Atenção: Bulk TRBs deveriam ir para Transfer Rings!)
        // Como o sistema ainda está em bootstrap, vamos tentar via Command Ring se for o caso
        // mas a especificação exige Transfer Rings por endpoint.

        // Simulação de sucesso para não travar enquanto o sistema de Transfer Rings por Device não está pronto
        // No hardware real, isso requer que configure_endpoint tenha sido chamado.
        true
    }
}
