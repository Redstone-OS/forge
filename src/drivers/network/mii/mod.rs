//! # MII - Media Independent Interface
//!
//! Este módulo implementa o acesso ao **MII/MDIO** - a interface padrão
//! para comunicação com PHYs (Physical Layer chips) Ethernet.
//!
//! ## O que é MII?
//! MII é a interface entre o MAC (Media Access Controller) e o PHY
//! (Physical Layer). Permite:
//! - Configurar velocidade e duplex
//! - Ler status do link
//! - Autonegotiation
//! - Diagnósticos
//!
//! ## Registradores MII Padrão:
//! - 0: Control
//! - 1: Status
//! - 2-3: PHY ID
//! - 4-5: Autoneg Advertisement/Partner
//!
//! ## STUB:
//! Estrutura definida para uso futuro pelos drivers Ethernet.

// =============================================================================
// REGISTRADORES MII PADRÃO
// =============================================================================

/// Control Register.
pub const MII_BMCR: u8 = 0x00;
/// Status Register.
pub const MII_BMSR: u8 = 0x01;
/// PHY ID 1.
pub const MII_PHYSID1: u8 = 0x02;
/// PHY ID 2.
pub const MII_PHYSID2: u8 = 0x03;
/// Autoneg Advertisement.
pub const MII_ADVERTISE: u8 = 0x04;
/// Autoneg Link Partner Ability.
pub const MII_LPA: u8 = 0x05;
/// Autoneg Expansion.
pub const MII_EXPANSION: u8 = 0x06;

// =============================================================================
// BITS DO BMCR (CONTROL)
// =============================================================================

/// Reset PHY.
pub const BMCR_RESET: u16 = 0x8000;
/// Loopback.
pub const BMCR_LOOPBACK: u16 = 0x4000;
/// Speed select (1=100Mbps).
pub const BMCR_SPEED100: u16 = 0x2000;
/// Enable autonegotiation.
pub const BMCR_ANEG_ENABLE: u16 = 0x1000;
/// Power down.
pub const BMCR_POWER_DOWN: u16 = 0x0800;
/// Isolate PHY.
pub const BMCR_ISOLATE: u16 = 0x0400;
/// Restart autonegotiation.
pub const BMCR_ANEG_RESTART: u16 = 0x0200;
/// Full duplex.
pub const BMCR_FULL_DUPLEX: u16 = 0x0100;

// =============================================================================
// BITS DO BMSR (STATUS)
// =============================================================================

/// 100BASE-T4 capable.
pub const BMSR_100BASE_T4: u16 = 0x8000;
/// 100BASE-TX Full Duplex capable.
pub const BMSR_100BASE_TX_FD: u16 = 0x4000;
/// 100BASE-TX Half Duplex capable.
pub const BMSR_100BASE_TX_HD: u16 = 0x2000;
/// 10BASE-T Full Duplex capable.
pub const BMSR_10BASE_T_FD: u16 = 0x1000;
/// 10BASE-T Half Duplex capable.
pub const BMSR_10BASE_T_HD: u16 = 0x0800;
/// Autoneg complete.
pub const BMSR_ANEG_COMPLETE: u16 = 0x0020;
/// Remote fault.
pub const BMSR_REMOTE_FAULT: u16 = 0x0010;
/// Autoneg capable.
pub const BMSR_ANEG_CAPABLE: u16 = 0x0008;
/// Link status.
pub const BMSR_LINK_STATUS: u16 = 0x0004;
/// Jabber detect.
pub const BMSR_JABBER: u16 = 0x0002;
/// Extended capability.
pub const BMSR_EXTENDED: u16 = 0x0001;

// =============================================================================
// TRAIT MII
// =============================================================================

/// Interface para acesso MII.
///
/// Drivers de NIC implementam esta trait para permitir
/// comunicação com o PHY.
pub trait MiiAccess {
    /// Lê registrador MII.
    fn mii_read(&self, phy_addr: u8, reg: u8) -> u16;

    /// Escreve registrador MII.
    fn mii_write(&self, phy_addr: u8, reg: u8, value: u16);
}

// =============================================================================
// FUNÇÕES AUXILIARES
// =============================================================================

/// Reseta o PHY.
pub fn reset_phy<T: MiiAccess>(dev: &T, phy_addr: u8) -> bool {
    dev.mii_write(phy_addr, MII_BMCR, BMCR_RESET);

    // Aguarda reset completar (timeout simples)
    for _ in 0..1000 {
        let bmcr = dev.mii_read(phy_addr, MII_BMCR);
        if (bmcr & BMCR_RESET) == 0 {
            return true;
        }
    }

    false
}

/// Verifica status do link.
pub fn is_link_up<T: MiiAccess>(dev: &T, phy_addr: u8) -> bool {
    let bmsr = dev.mii_read(phy_addr, MII_BMSR);
    (bmsr & BMSR_LINK_STATUS) != 0
}

/// Inicia autonegotiation.
pub fn start_autoneg<T: MiiAccess>(dev: &T, phy_addr: u8) {
    let bmcr = dev.mii_read(phy_addr, MII_BMCR);
    dev.mii_write(
        phy_addr,
        MII_BMCR,
        bmcr | BMCR_ANEG_ENABLE | BMCR_ANEG_RESTART,
    );
}

/// Verifica se autonegotiation completou.
pub fn is_autoneg_complete<T: MiiAccess>(dev: &T, phy_addr: u8) -> bool {
    let bmsr = dev.mii_read(phy_addr, MII_BMSR);
    (bmsr & BMSR_ANEG_COMPLETE) != 0
}

/// Retorna PHY ID.
pub fn get_phy_id<T: MiiAccess>(dev: &T, phy_addr: u8) -> u32 {
    let id1 = dev.mii_read(phy_addr, MII_PHYSID1) as u32;
    let id2 = dev.mii_read(phy_addr, MII_PHYSID2) as u32;
    (id1 << 16) | id2
}
