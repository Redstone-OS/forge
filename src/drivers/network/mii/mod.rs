//! # MII (Media Independent Interface) Management
//!
//! Este módulo fornece utilitários para interagir com chips PHY através
//! da interface de gerenciamento MII/MDIO.

pub mod regs;

use self::regs::*;

/// Abstração de um chip PHY Ethernet
pub struct PhyDevice<T: MiiBus> {
    bus: T,
    addr: u8,
}

/// Trait para barramentos que suportam leitura/escrita MII (ex: e1000, rtl8139)
pub trait MiiBus {
    fn read(&self, addr: u8, reg: u16) -> u16;
    fn write(&self, addr: u8, reg: u16, val: u16);
}

impl<T: MiiBus> PhyDevice<T> {
    pub fn new(bus: T, addr: u8) -> Self {
        Self { bus, addr }
    }

    /// STUB: Inicialização do PHY
    pub fn init(&self) {
        // 1. Resetar o PHY via BMCR::RESET
        // 2. Aguardar a conclusão do reset
        // 3. Iniciar Auto-negociação (AN_RESTART)
        // 4. Verificar Link Status periodicamente
        crate::kdebug!(
            "(Net/MII) PHY em addr {} inicializado (stub).",
            self.addr as u64
        );
    }

    /// STUB: Verifica se o link físico está estabelecido
    pub fn is_link_up(&self) -> bool {
        let status = self.bus.read(self.addr, regs::BMSR);
        (status & bmsr::LINK_STATUS) != 0
    }

    /// STUB: Reinicia a auto-negociação
    pub fn restart_an(&self) {
        let mut ctrl = self.bus.read(self.addr, regs::BMCR);
        ctrl |= bmcr::AN_ENABLE | bmcr::AN_RESTART;
        self.bus.write(self.addr, regs::BMCR, ctrl);
    }
}
