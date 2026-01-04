//! # MII (Media Independent Interface) Standard Registers
//!
//! Registradores padrão definidos pelo IEEE 802.3 para comunicação entre MAC e PHY.

/// Registradores de Gerenciamento MII
pub mod regs {
    /// Basic Mode Control Register
    pub const BMCR: u16 = 0x00;
    /// Basic Mode Status Register
    pub const BMSR: u16 = 0x01;
    /// PHY Identifier 1
    pub const PHYID1: u16 = 0x02;
    /// PHY Identifier 2
    pub const PHYID2: u16 = 0x03;
    /// Auto-Negotiation Advertisement
    pub const ANAR: u16 = 0x04;
    /// Auto-Negotiation Link Partner Ability
    pub const ANLPAR: u16 = 0x05;
    /// Auto-Negotiation Expansion
    pub const ANER: u16 = 0x06;
}

/// Bits do BMCR (Control)
pub mod bmcr {
    pub const RESET: u16 = 1 << 15;
    pub const LOOPBACK: u16 = 1 << 14;
    pub const SPEED_100: u16 = 1 << 13;
    pub const AN_ENABLE: u16 = 1 << 12;
    pub const POWER_DOWN: u16 = 1 << 11;
    pub const ISOLATE: u16 = 1 << 10;
    pub const AN_RESTART: u16 = 1 << 9;
    pub const FULL_DUPLEX: u16 = 1 << 8;
    pub const COLLISION_TEST: u16 = 1 << 7;
}

/// Bits do BMSR (Status)
pub mod bmsr {
    pub const S100BASE_T4: u16 = 1 << 15;
    pub const S100BASE_TX_FD: u16 = 1 << 14;
    pub const S100BASE_TX_HD: u16 = 1 << 13;
    pub const S10BASE_T_FD: u16 = 1 << 12;
    pub const S10BASE_T_HD: u16 = 1 << 11;
    pub const AN_COMPLETE: u16 = 1 << 5;
    pub const REMOTE_FAULT: u16 = 1 << 4;
    pub const AN_ABILITY: u16 = 1 << 3;
    pub const LINK_STATUS: u16 = 1 << 2;
    pub const JABBER_DETECT: u16 = 1 << 1;
    pub const EXTENDED_CAP: u16 = 1 << 0;
}
