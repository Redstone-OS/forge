//! # CPU Topology
//!
//! Gerenciamento da topologia de processadores do sistema.
//!
//! ## Responsabilidades
//!
//! - Manter registro de todas as CPUs detectadas via ACPI
//! - Mapear APIC ID (hardware) → Logical ID (0 a N-1)
//! - Fornecer `cpu_count()` para alocações dinâmicas
//!
//! ## Estrutura de Dados
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │                    TOPOLOGY                          │
//! │                                                      │
//! │  cpus: [CpuInfo; MAX_CPUS]                           │
//! │  ┌─────┬─────┬─────┬─────┬─────┬─────┐               │
//! │  │ CPU0│ CPU1│ CPU2│ CPU3│ ... │None │               │
//! │  │ BSP │ AP  │ AP  │ AP  │     │     │               │
//! │  └─────┴─────┴─────┴─────┴─────┴─────┘               │
//! │                                                      │
//! │  count: 4                                            │
//! │  bsp_logical_id: 0                                   │
//! │  initialized: true                                   │
//! └──────────────────────────────────────────────────────┘
//! ```

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Número máximo de CPUs suportadas
pub const MAX_CPUS: usize = 256;

/// Informações sobre uma CPU
#[derive(Debug, Clone, Copy)]
pub struct CpuInfo {
    /// ID lógico (0 a N-1, atribuído pelo kernel)
    pub logical_id: u32,
    /// APIC ID (hardware)
    pub apic_id: u32,
    /// ACPI Processor ID
    pub acpi_id: u32,
    /// É o Bootstrap Processor?
    pub is_bsp: bool,
    /// CPU está online?
    pub online: bool,
}

impl CpuInfo {
    const fn empty() -> Self {
        Self {
            logical_id: 0,
            apic_id: 0,
            acpi_id: 0,
            is_bsp: false,
            online: false,
        }
    }
}

/// Topologia global de CPUs
pub struct CpuTopology {
    /// Array de CPUs (índice = logical_id)
    cpus: [CpuInfo; MAX_CPUS],
    /// Número de CPUs detectadas
    count: AtomicUsize,
    /// Logical ID do BSP
    bsp_id: AtomicUsize,
    /// Inicializado?
    initialized: AtomicBool,
}

impl CpuTopology {
    /// Cria topologia vazia
    const fn new() -> Self {
        const EMPTY: CpuInfo = CpuInfo::empty();
        Self {
            cpus: [EMPTY; MAX_CPUS],
            count: AtomicUsize::new(0),
            bsp_id: AtomicUsize::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    /// Inicializa a topologia com dados do ACPI
    ///
    /// # Safety
    ///
    /// Deve ser chamado apenas uma vez, após parsing do MADT.
    pub unsafe fn init(&mut self, acpi_cpus: &[Option<crate::arch::x86_64::acpi::madt::CpuEntry>]) {
        let mut count = 0;
        let mut bsp_id = 0;

        for cpu_opt in acpi_cpus.iter() {
            if let Some(cpu) = cpu_opt {
                if count >= MAX_CPUS {
                    crate::kwarn!("(SMP) Mais CPUs que MAX_CPUS, ignorando extras");
                    break;
                }

                self.cpus[count] = CpuInfo {
                    logical_id: count as u32,
                    apic_id: cpu.apic_id as u32,
                    acpi_id: cpu.acpi_id as u32,
                    is_bsp: cpu.is_bsp,
                    online: cpu.is_bsp, // Apenas BSP começa online
                };

                if cpu.is_bsp {
                    bsp_id = count;
                }

                count += 1;
            }
        }

        // Garantir pelo menos 1 CPU
        if count == 0 {
            self.cpus[0] = CpuInfo {
                logical_id: 0,
                apic_id: 0,
                acpi_id: 0,
                is_bsp: true,
                online: true,
            };
            count = 1;
        }

        self.count.store(count, Ordering::Release);
        self.bsp_id.store(bsp_id, Ordering::Release);
        self.initialized.store(true, Ordering::Release);

        crate::kinfo!("(SMP/Topology) Inicializado:", count as u64, "CPUs");
    }

    /// Retorna o número de CPUs
    #[inline]
    pub fn count(&self) -> usize {
        self.count.load(Ordering::Acquire)
    }

    /// Retorna informações de uma CPU por logical ID
    #[inline]
    pub fn get(&self, logical_id: usize) -> Option<&CpuInfo> {
        if logical_id < self.count.load(Ordering::Acquire) {
            Some(&self.cpus[logical_id])
        } else {
            None
        }
    }

    /// Encontra CPU pelo APIC ID
    pub fn find_by_apic_id(&self, apic_id: u32) -> Option<&CpuInfo> {
        let count = self.count.load(Ordering::Acquire);
        for i in 0..count {
            if self.cpus[i].apic_id == apic_id {
                return Some(&self.cpus[i]);
            }
        }
        None
    }

    /// Retorna o logical ID do BSP
    #[inline]
    pub fn bsp_id(&self) -> usize {
        self.bsp_id.load(Ordering::Acquire)
    }

    /// Marca uma CPU como online
    pub fn set_online(&mut self, logical_id: usize) {
        if logical_id < MAX_CPUS {
            self.cpus[logical_id].online = true;
        }
    }

    /// Itera sobre CPUs válidas
    pub fn iter(&self) -> impl Iterator<Item = &CpuInfo> {
        let count = self.count.load(Ordering::Acquire);
        self.cpus[..count].iter()
    }

    /// Itera sobre APs (Application Processors, exclui BSP)
    pub fn iter_aps(&self) -> impl Iterator<Item = &CpuInfo> {
        self.iter().filter(|cpu| !cpu.is_bsp)
    }

    /// Está inicializado?
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }
}

// =============================================================================
// Global Instance
// =============================================================================

/// Topologia global de CPUs
///
/// Acessível após `topology::init()` ser chamado.
pub static mut TOPOLOGY: CpuTopology = CpuTopology::new();

// =============================================================================
// API Pública
// =============================================================================

/// Inicializa a topologia com dados do ACPI
pub fn init(acpi_cpus: &[Option<crate::arch::x86_64::acpi::madt::CpuEntry>]) {
    unsafe {
        TOPOLOGY.init(acpi_cpus);
    }
}

/// Retorna o número de CPUs detectadas
///
/// # Panics
///
/// Retorna 1 se a topologia não foi inicializada.
#[inline]
pub fn cpu_count() -> usize {
    unsafe {
        if TOPOLOGY.is_initialized() {
            TOPOLOGY.count()
        } else {
            1 // Fallback: apenas BSP
        }
    }
}

/// Retorna informações da CPU atual
#[inline]
pub fn current_cpu() -> &'static CpuInfo {
    let apic_id = crate::arch::x86_64::apic::lapic::id();
    unsafe {
        TOPOLOGY
            .find_by_apic_id(apic_id)
            .unwrap_or(&TOPOLOGY.cpus[0])
    }
}

/// Retorna o logical ID da CPU atual
#[inline]
pub fn current_cpu_id() -> usize {
    current_cpu().logical_id as usize
}

/// Retorna referência à topologia global
pub fn get() -> &'static CpuTopology {
    unsafe { &*core::ptr::addr_of!(TOPOLOGY) }
}

/// Retorna referência mutável à topologia (para set_online)
pub unsafe fn get_mut() -> &'static mut CpuTopology {
    &mut *core::ptr::addr_of_mut!(TOPOLOGY)
}
