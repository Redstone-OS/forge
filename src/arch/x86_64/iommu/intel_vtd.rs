#![allow(dead_code)]
/// Arquivo: x86_64/iommu/intel_vtd.rs
///
/// Propósito: Implementação do suporte a Intel VT-d (Virtualization Technology for Directed I/O).
/// Responsável por configurar DMA Remapping.
///
/// Detalhes de Implementação:
/// - Define offsets de registradores MMIO da IOMMU.
/// - Implementa sequência de inicialização: Setup Root Table -> Enable Translation.
/// - Estruturas de tabelas de remapeamento.

/// Intel VT-d (DMA Remapping)
use core::ptr::{read_volatile, write_volatile};

// --- Registradores MMIO (Offsets do Base Address) ---
const DMAR_VER_REG: usize = 0x00; // Version Register
const DMAR_CAP_REG: usize = 0x08; // Capability Register
const DMAR_ECAP_REG: usize = 0x10; // Extended Capability Register
const DMAR_GCMD_REG: usize = 0x18; // Global Command Register
const DMAR_GSTS_REG: usize = 0x1C; // Global Status Register
const DMAR_RTADDR_REG: usize = 0x20; // Root Table Address Register

// --- Bits do Global Command Register ---
const GCMD_TE: u32 = 1 << 31; // Translation Enable
const GCMD_SRTP: u32 = 1 << 30; // Set Root Table Pointer

// --- Bits do Global Status Register ---
const GSTS_TES: u32 = 1 << 31; // Translation Enable Status
const GSTS_RTPS: u32 = 1 << 30; // Root Table Pointer Status

/// Estrutura para entrada na Root Table (128 bits)
/// Mapeia um Bus Number (0-255) para uma Context Table.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct RootEntry {
    pub lower: u64, // Present (0), Context Table Pointer (12-63)
    pub upper: u64, // Reservado
}

impl RootEntry {
    pub const fn new() -> Self {
        Self { lower: 0, upper: 0 }
    }

    pub fn set_present(&mut self, context_table_phys: u64) {
        // Bit 0 = Present, Bits 12-63 = Endereço Físico (4K aligned)
        self.lower = (context_table_phys & !0xFFF) | 1;
    }
}

/// Acessador de Registradores MMIO
struct DmarRegisters {
    base: *mut u8,
}

impl DmarRegisters {
    unsafe fn new(base: u64) -> Self {
        Self {
            base: base as *mut u8,
        }
    }

    unsafe fn read_u32(&self, offset: usize) -> u32 {
        read_volatile(self.base.add(offset) as *const u32)
    }

    unsafe fn write_u32(&self, offset: usize, value: u32) {
        write_volatile(self.base.add(offset) as *mut u32, value)
    }

    unsafe fn write_u64(&self, offset: usize, value: u64) {
        write_volatile(self.base.add(offset) as *mut u64, value)
    }

    // Espera um bit no status register ficar (set=true) ou (set=false)
    unsafe fn wait_gsts(&self, bit: u32, set: bool) {
        loop {
            let status = self.read_u32(DMAR_GSTS_REG);
            if set {
                if (status & bit) != 0 {
                    break;
                }
            } else {
                if (status & bit) == 0 {
                    break;
                }
            }
            crate::arch::x86_64::cpu::Cpu::halt(); // Relaxa a CPU
                                                   // Em loop de polling real, deve ter timeout
        }
    }
}

/// Inicializa o subsistema Intel VT-d para uma unidade DMAR específica.
///
/// # Argumentos
///
/// * `dmar_base_address`: Endereço físico base da unidade (MMIO).
/// * `root_table_phys`: Endereço físico da Root Table pré-alocada (4KB aligned).
///
/// # Safety
///
/// Escreve em MMIO. O caller deve garantir que `root_table_phys` é válido e contém
/// uma tabela vazia (zerada) ou configurada corretamente.
pub unsafe fn init(dmar_base_address: u64, root_table_phys: u64) {
    let regs = DmarRegisters::new(dmar_base_address);

    // 1. Ler Versão (apenas para debug/verificação)
    let _ver = regs.read_u32(DMAR_VER_REG);

    // 2. Desabilitar Translation (se estiver habilitado por firmware)
    // Escrever TE=0 no GCMD não limpa imediatamente, precisa checar GSTS
    // Mas geralmente iniciamos com reset. Vamos assumir estado limpo ou desligar.

    // 3. Configurar Root Table Pointer
    regs.write_u64(DMAR_RTADDR_REG, root_table_phys);

    // 4. Latch Root Table Pointer (GCMD_SRTP)
    // Escreve bit SRTP no Global Command
    let mut cmd = regs.read_u32(DMAR_GCMD_REG);
    regs.write_u32(DMAR_GCMD_REG, cmd | GCMD_SRTP);

    // Esperar SRTP ser refletido no Status (RTPS)
    regs.wait_gsts(GSTS_RTPS, true);

    // 5. Habilitar Translation (TE)
    // IMPORTANTE: Só habilitar depois de configurar tabelas de remapeamento válidas
    // ou ter uma estratégia de identidade (pass-through).
    // Se habilitarmos com tabela vazia (Present=0), todo DMA será bloqueado (bom para segurança default).
    cmd = regs.read_u32(DMAR_GCMD_REG);
    regs.write_u32(DMAR_GCMD_REG, cmd | GCMD_TE);

    // Esperar TES (Translation Enable Status)
    regs.wait_gsts(GSTS_TES, true);

    // Agora o VT-d está ativo e filtrando DMA baseado na Root Table.
}
