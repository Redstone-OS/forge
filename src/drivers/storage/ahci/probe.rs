//! # Probe de Portas AHCI
//!
//! Funções para detectar e identificar dispositivos AHCI.

use super::regs::{
    cap, det, ghc, hba, ipm, port, port_cmd, port_ssts, signatures, PORT_BASE, PORT_SIZE,
};
use crate::mm::VirtAddr;

/// Informações sobre uma porta AHCI detectada
#[derive(Debug, Clone, Copy)]
pub struct AhciPort {
    /// Índice da porta (0-31)
    pub index: u8,
    /// Tipo de dispositivo conectado
    pub device_type: AhciDeviceType,
    /// Signature do dispositivo
    pub signature: u32,
}

/// Tipo de dispositivo AHCI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AhciDeviceType {
    /// Nenhum dispositivo
    None,
    /// Disco SATA
    Sata,
    /// Dispositivo ATAPI (CD/DVD)
    Atapi,
    /// Enclosure Management Bridge
    Semb,
    /// Port Multiplier
    PortMultiplier,
    /// Desconhecido
    Unknown,
}

impl AhciDeviceType {
    /// Converte uma signature para o tipo de dispositivo
    pub fn from_signature(sig: u32) -> Self {
        match sig {
            signatures::SATA_ATA => AhciDeviceType::Sata,
            signatures::SATA_ATAPI => AhciDeviceType::Atapi,
            signatures::SATA_SEMB => AhciDeviceType::Semb,
            signatures::SATA_PM => AhciDeviceType::PortMultiplier,
            _ => AhciDeviceType::Unknown,
        }
    }

    /// Retorna nome legível do tipo
    pub fn name(&self) -> &'static str {
        match self {
            AhciDeviceType::None => "None",
            AhciDeviceType::Sata => "SATA",
            AhciDeviceType::Atapi => "ATAPI",
            AhciDeviceType::Semb => "SEMB",
            AhciDeviceType::PortMultiplier => "Port Multiplier",
            AhciDeviceType::Unknown => "Unknown",
        }
    }
}

/// Lê um registrador MMIO de 32 bits
#[inline]
unsafe fn read_reg32(base: VirtAddr, offset: u64) -> u32 {
    let addr = base.as_u64() + offset;
    core::ptr::read_volatile(addr as *const u32)
}

/// Escreve em um registrador MMIO de 32 bits
#[inline]
#[allow(dead_code)]
unsafe fn write_reg32(base: VirtAddr, offset: u64, value: u32) {
    let addr = base.as_u64() + offset;
    core::ptr::write_volatile(addr as *mut u32, value);
}

/// Verifica se o controlador suporta AHCI e está habilitado
pub fn check_ahci_enabled(abar: VirtAddr) -> bool {
    let ghc_val = unsafe { read_reg32(abar, hba::GHC) };
    (ghc_val & ghc::AE) != 0
}

/// Lê as capabilities do controlador
pub fn read_capabilities(abar: VirtAddr) -> (u32, u8, u8) {
    let cap_val = unsafe { read_reg32(abar, hba::CAP) };

    // Número de portas implementadas (bits 4:0) + 1
    let num_ports = ((cap_val & cap::NP_MASK) + 1) as u8;

    // Número de command slots (bits 12:8) + 1
    let num_cmd_slots = (((cap_val & cap::NCS_MASK) >> cap::NCS_SHIFT) + 1) as u8;

    (cap_val, num_ports, num_cmd_slots)
}

/// Lê a versão AHCI
pub fn read_version(abar: VirtAddr) -> (u16, u16) {
    let vs = unsafe { read_reg32(abar, hba::VS) };
    let major = ((vs >> 16) & 0xFFFF) as u16;
    let minor = (vs & 0xFFFF) as u16;
    (major, minor)
}

/// Lê a máscara de portas implementadas
pub fn read_ports_implemented(abar: VirtAddr) -> u32 {
    unsafe { read_reg32(abar, hba::PI) }
}

/// Verifica o status de uma porta específica
pub fn probe_port(abar: VirtAddr, port_num: u8) -> Option<AhciPort> {
    let port_base = PORT_BASE + (port_num as u64) * PORT_SIZE;

    // Ler status da porta
    let ssts = unsafe { read_reg32(abar, port_base + port::SSTS) };

    // Verificar Device Detection
    let det_val = ssts & port_ssts::DET_MASK;
    let ipm_val = (ssts & port_ssts::IPM_MASK) >> port_ssts::IPM_SHIFT;

    // Verificar se há dispositivo presente e ativo
    if det_val != det::COMM || ipm_val != ipm::ACTIVE {
        return None;
    }

    // Ler signature do dispositivo
    let sig = unsafe { read_reg32(abar, port_base + port::SIG) };
    let device_type = AhciDeviceType::from_signature(sig);

    Some(AhciPort {
        index: port_num,
        device_type,
        signature: sig,
    })
}

/// Escaneia todas as portas e retorna as que têm dispositivos conectados
pub fn scan_ports(abar: VirtAddr) -> alloc::vec::Vec<AhciPort> {
    let mut found_ports = alloc::vec::Vec::new();

    let pi = read_ports_implemented(abar);

    for i in 0..32u8 {
        // Verificar se a porta está implementada
        if (pi & (1 << i)) == 0 {
            continue;
        }

        if let Some(port_info) = probe_port(abar, i) {
            found_ports.push(port_info);
        }
    }

    found_ports
}

/// Verifica se uma porta está pronta para processar comandos
#[allow(dead_code)]
pub fn is_port_idle(abar: VirtAddr, port_num: u8) -> bool {
    let port_base = PORT_BASE + (port_num as u64) * PORT_SIZE;
    let cmd = unsafe { read_reg32(abar, port_base + port::CMD) };

    // Verificar se ST, FRE, CR e FR estão todos desligados
    (cmd & (port_cmd::ST | port_cmd::FRE | port_cmd::CR | port_cmd::FR)) == 0
}
