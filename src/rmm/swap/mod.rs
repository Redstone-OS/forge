//! # Swap Subsystem
//!
//! Swap de páginas anônimas para armazenamento secundário.
//!
//! ## Conceito
//!
//! Quando a memória RAM fica escassa, páginas anônimas (heap, stack)
//! podem ser escritas em disco e liberadas. Quando acessadas novamente,
//! são trazidas de volta (page fault → swap in).
//!
//! ## Arquitetura
//!
//! ```text
//! ┌───────────────────────────────────────────────────────────┐
//! │                      Swap Subsystem                       │
//! ├───────────────────────────────────────────────────────────┤
//! │                                                           │
//! │  ┌─────────────────────────────────────────────────────┐  │
//! │  │                   SwapDevice                        │  │
//! │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐    │  │
//! │  │  │ Slot 0  │ │ Slot 1  │ │ Slot 2  │ │   ...   │    │  │
//! │  │  │  4KB    │ │  4KB    │ │  4KB    │ │         │    │  │
//! │  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘    │  │
//! │  └─────────────────────────────────────────────────────┘  │
//! │         │                                                 │
//! │         ▼                                                 │
//! │  ┌─────────────────────────────────────────────────────┐  │
//! │  │              SlotAllocator (Bitmap)                 │  │
//! │  │  [1][1][0][0][1][0][0][0]...                        │  │
//! │  │   ▲     ▲                                           │  │
//! │  │   │     └── free                                    │  │
//! │  │   └── used                                          │  │
//! │  └─────────────────────────────────────────────────────┘  │
//! │                                                           │
//! │ Page Fault ──► swap_in() ──► Read from disk ──► Map page  │
//! │ Reclaim    ──► swap_out() ──► Write to disk ──► Free page │
//! └───────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Swap Entry no PTE
//!
//! Quando uma página está em swap, o PTE contém:
//! ```text
//! ┌───────────────────────────────────────────────────────────┐
//! │ Bit 0 = 0 (not present)                                   │
//! │ Bits 1-7: Swap type (qual device)                         │
//! │ Bits 8-63: Slot offset                                    │
//! └───────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Dependências
//!
//! - Block device (para I/O)
//! - Page reclaim (dispara swap_out)
//! - Page fault handler (dispara swap_in)

pub mod device;
pub mod entry;
pub mod slot;

pub use device::{SwapDevice, SwapDeviceOps};
pub use entry::SwapEntry;
pub use slot::{SlotAllocator, SwapSlot};

use crate::rmm::addr::PhysAddr;
use crate::rmm::config::PAGE_SIZE;
use crate::sync::Spinlock;

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// =============================================================================
// Estado Global
// =============================================================================

/// Estado global do swap
static SWAP_STATE: Spinlock<SwapState> = Spinlock::new(SwapState::new());

/// Flag indicando se swap está habilitado
static SWAP_ENABLED: AtomicBool = AtomicBool::new(false);

/// Contador de páginas em swap
static PAGES_IN_SWAP: AtomicU64 = AtomicU64::new(0);

/// Contador de swap-ins
static SWAP_INS: AtomicU64 = AtomicU64::new(0);

/// Contador de swap-outs
static SWAP_OUTS: AtomicU64 = AtomicU64::new(0);

// =============================================================================
// SwapState
// =============================================================================

/// Estado interno do swap
struct SwapState {
    /// Devices de swap registrados
    devices: [Option<SwapDevice>; MAX_SWAP_DEVICES],
    /// Número de devices ativos
    device_count: usize,
    /// Total de slots disponíveis
    total_slots: u64,
    /// Slots em uso
    used_slots: u64,
}

const MAX_SWAP_DEVICES: usize = 4;

impl SwapState {
    const fn new() -> Self {
        Self {
            devices: [None, None, None, None],
            device_count: 0,
            total_slots: 0,
            used_slots: 0,
        }
    }
}

// =============================================================================
// API Pública
// =============================================================================

/// Inicializa subsistema de swap
pub fn init() {
    crate::kinfo!("(RMM/Swap) Inicializando subsistema de swap...");

    // Swap começa desabilitado até um device ser adicionado
    SWAP_ENABLED.store(false, Ordering::Release);

    crate::kinfo!("(RMM/Swap) Aguardando configuração de dispositivo");
}

/// Adiciona dispositivo de swap
pub fn add_device(device: SwapDevice) -> Result<usize, SwapError> {
    let mut state = SWAP_STATE.lock();

    if state.device_count >= MAX_SWAP_DEVICES {
        return Err(SwapError::TooManyDevices);
    }

    let idx = state.device_count;
    let slots = device.total_slots();

    state.devices[idx] = Some(device);
    state.device_count += 1;
    state.total_slots += slots;

    SWAP_ENABLED.store(true, Ordering::Release);

    crate::kinfo!(
        "(RMM/Swap) Device {} adicionado: {} slots ({} MB)",
        idx,
        slots,
        (slots * PAGE_SIZE as u64) / (1024 * 1024)
    );

    Ok(idx)
}

/// Remove dispositivo de swap
pub fn remove_device(idx: usize) -> Result<(), SwapError> {
    let mut state = SWAP_STATE.lock();

    if idx >= state.device_count {
        return Err(SwapError::InvalidDevice);
    }

    if let Some(device) = &state.devices[idx] {
        if device.used_slots() > 0 {
            return Err(SwapError::DeviceBusy);
        }

        state.total_slots -= device.total_slots();
    }

    state.devices[idx] = None;

    // Reabilita flag se não há mais devices
    if state.devices.iter().all(|d| d.is_none()) {
        SWAP_ENABLED.store(false, Ordering::Release);
    }

    Ok(())
}

/// Escreve página para swap (swap out)
///
/// Retorna entrada de swap que pode ser armazenada no PTE.
pub fn swap_out(phys: PhysAddr) -> Result<SwapEntry, SwapError> {
    if !is_enabled() {
        return Err(SwapError::NotEnabled);
    }

    let mut state = SWAP_STATE.lock();

    // Encontra device com espaço livre
    for (dev_idx, device_opt) in state.devices.iter_mut().enumerate() {
        if let Some(device) = device_opt {
            if let Some(slot) = device.alloc_slot() {
                // Escreve página no slot
                if let Err(e) = device.write_page(slot, phys) {
                    device.free_slot(slot);
                    return Err(e);
                }

                state.used_slots += 1;
                drop(state);

                PAGES_IN_SWAP.fetch_add(1, Ordering::Relaxed);
                SWAP_OUTS.fetch_add(1, Ordering::Relaxed);

                return Ok(SwapEntry::new(dev_idx as u8, slot));
            }
        }
    }

    Err(SwapError::NoSpace)
}

/// Lê página de swap (swap in)
///
/// Retorna endereço físico da nova página alocada.
pub fn swap_in(entry: SwapEntry) -> Result<PhysAddr, SwapError> {
    if !is_enabled() {
        return Err(SwapError::NotEnabled);
    }

    let mut state = SWAP_STATE.lock();

    let dev_idx = entry.device() as usize;
    if dev_idx >= state.device_count {
        return Err(SwapError::InvalidDevice);
    }

    if let Some(device) = &mut state.devices[dev_idx] {
        let slot = entry.slot();

        // Aloca nova página física
        let phys = crate::rmm::phys::alloc(
            crate::rmm::phys::FrameOwner::Kernel,
            crate::rmm::zone::Zone::Normal,
            crate::rmm::phys::AllocFlags::ZERO,
        )
        .ok_or(SwapError::OutOfMemory)?;

        // Lê do swap
        if let Err(e) = device.read_page(slot, phys) {
            // Libera página alocada
            let _ = crate::rmm::phys::free(phys, crate::rmm::phys::FrameOwner::Kernel);
            return Err(e);
        }

        // Libera slot
        device.free_slot(slot);
        state.used_slots -= 1;
        drop(state);

        PAGES_IN_SWAP.fetch_sub(1, Ordering::Relaxed);
        SWAP_INS.fetch_add(1, Ordering::Relaxed);

        return Ok(phys);
    }

    Err(SwapError::InvalidDevice)
}

/// Libera entrada de swap sem ler o conteúdo
pub fn free_entry(entry: SwapEntry) -> Result<(), SwapError> {
    let mut state = SWAP_STATE.lock();

    let dev_idx = entry.device() as usize;
    if dev_idx >= state.device_count {
        return Err(SwapError::InvalidDevice);
    }

    if let Some(device) = &mut state.devices[dev_idx] {
        device.free_slot(entry.slot());
        state.used_slots -= 1;

        PAGES_IN_SWAP.fetch_sub(1, Ordering::Relaxed);
    }

    Ok(())
}

/// Verifica se swap está habilitado
#[inline]
pub fn is_enabled() -> bool {
    SWAP_ENABLED.load(Ordering::Acquire)
}

/// Retorna espaço usado em páginas
pub fn used() -> u64 {
    SWAP_STATE.lock().used_slots
}

/// Retorna espaço total em páginas
pub fn total() -> u64 {
    SWAP_STATE.lock().total_slots
}

/// Retorna espaço livre em páginas
pub fn free() -> u64 {
    let state = SWAP_STATE.lock();
    state.total_slots.saturating_sub(state.used_slots)
}

// =============================================================================
// Estatísticas
// =============================================================================

/// Estatísticas do swap
#[derive(Debug, Clone, Copy, Default)]
pub struct SwapStats {
    pub enabled: bool,
    pub total_slots: u64,
    pub used_slots: u64,
    pub free_slots: u64,
    pub swap_ins: u64,
    pub swap_outs: u64,
    pub device_count: usize,
}

/// Retorna estatísticas do swap
pub fn stats() -> SwapStats {
    let state = SWAP_STATE.lock();

    SwapStats {
        enabled: SWAP_ENABLED.load(Ordering::Relaxed),
        total_slots: state.total_slots,
        used_slots: state.used_slots,
        free_slots: state.total_slots.saturating_sub(state.used_slots),
        swap_ins: SWAP_INS.load(Ordering::Relaxed),
        swap_outs: SWAP_OUTS.load(Ordering::Relaxed),
        device_count: state.device_count,
    }
}

/// Dump estatísticas
pub fn dump_stats() {
    let s = stats();
    crate::kinfo!("=== Swap Statistics ===");
    crate::kinfo!("  Enabled: {}", s.enabled);
    crate::kinfo!("  Devices: {}", s.device_count);
    crate::kinfo!(
        "  Slots: {} used / {} total ({} free)",
        s.used_slots,
        s.total_slots,
        s.free_slots
    );
    crate::kinfo!("  Swap-ins: {}, Swap-outs: {}", s.swap_ins, s.swap_outs);

    let used_mb = (s.used_slots * PAGE_SIZE as u64) / (1024 * 1024);
    let total_mb = (s.total_slots * PAGE_SIZE as u64) / (1024 * 1024);
    crate::kinfo!("  Size: {} MB used / {} MB total", used_mb, total_mb);
}

// =============================================================================
// Erros
// =============================================================================

/// Erros do subsistema de swap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapError {
    /// Swap não está habilitado
    NotEnabled,
    /// Sem espaço no swap
    NoSpace,
    /// Dispositivo inválido
    InvalidDevice,
    /// Slot inválido
    InvalidSlot,
    /// Erro de I/O
    IoError,
    /// Muitos dispositivos
    TooManyDevices,
    /// Dispositivo ocupado
    DeviceBusy,
    /// Sem memória física
    OutOfMemory,
}

impl core::fmt::Display for SwapError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotEnabled => write!(f, "swap not enabled"),
            Self::NoSpace => write!(f, "no space in swap"),
            Self::InvalidDevice => write!(f, "invalid swap device"),
            Self::InvalidSlot => write!(f, "invalid swap slot"),
            Self::IoError => write!(f, "swap I/O error"),
            Self::TooManyDevices => write!(f, "too many swap devices"),
            Self::DeviceBusy => write!(f, "swap device busy"),
            Self::OutOfMemory => write!(f, "out of memory"),
        }
    }
}
