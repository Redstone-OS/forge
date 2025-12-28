# Guia de Revisão: `module/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE.

---

## 1. PROPÓSITO

Sistema de carregamento seguro de módulos (drivers) em Ring 0 supervisionado.

---

## 2. ESTRUTURA

```
module/
├── mod.rs              ✅ JÁ EXISTE
├── abi.rs              → Interface estável
├── capability.rs       → Capabilities de módulo
├── loader.rs           → Parser ELF
├── sandbox.rs          → Isolamento
├── supervisor.rs       → Ciclo de vida
├── verifier.rs         → Verificação de assinatura
└── watchdog.rs         → Monitoramento de saúde
```

---

## 3. REGRAS

### ❌ NUNCA:
- Confiar em módulos (mesmo assinados)
- Dar acesso direto a page tables
- Permitir DMA sem IOMMU
- Permitir módulo carregar outros

### ✅ SEMPRE:
- Verificar assinatura antes de carregar
- Usar timeout em inicialização
- Monitorar saúde via watchdog
- Revogar cap on fault

---

## 4. IMPLEMENTAÇÕES

### 4.1 `abi.rs`

```rust
//! ABI estável para módulos

/// Versão da ABI
pub const ABI_VERSION: u32 = 1;

/// Magic number para validação
pub const MODULE_MAGIC: u32 = 0x4D4F4452; // "MODR"

/// Informações do módulo (header no binário)
#[repr(C)]
pub struct ModuleInfo {
    /// Magic number
    pub magic: u32,
    /// Versão da ABI
    pub abi_version: u32,
    /// Nome do módulo (null-terminated)
    pub name: [u8; 32],
    /// Versão do módulo
    pub version: u32,
    /// Flags
    pub flags: u32,
    /// Capabilities requisitadas (bitmask)
    pub required_caps: u64,
}

/// ABI de callbacks do módulo
#[repr(C)]
pub struct ModuleAbi {
    /// Chamado para inicializar
    pub init: Option<extern "C" fn() -> i32>,
    /// Chamado para cleanup
    pub cleanup: Option<extern "C" fn()>,
    /// Chamado por healthcheck
    pub health: Option<extern "C" fn() -> i32>,
}

impl ModuleInfo {
    /// Verifica se é válido
    pub fn is_valid(&self) -> bool {
        self.magic == MODULE_MAGIC && self.abi_version == ABI_VERSION
    }
}
```

### 4.2 `capability.rs`

```rust
//! Capabilities específicas de módulos

/// Tipos de capability que módulos podem requisitar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum ModuleCapType {
    /// Acesso a DMA (requer IOMMU)
    DmaAccess = 1 << 0,
    /// Registrar handler de IRQ
    IrqHandler = 1 << 1,
    /// Acessar região MMIO específica
    MmioAccess = 1 << 2,
    /// Acessar portas IO
    IoPortAccess = 1 << 3,
    /// Alocar memória física contígua
    PhysAlloc = 1 << 4,
    /// Registrar block device
    BlockDevice = 1 << 5,
    /// Registrar char device
    CharDevice = 1 << 6,
    /// Registrar network device
    NetDevice = 1 << 7,
    /// Acessar config PCI
    PciConfig = 1 << 8,
}

/// Capability concedida a módulo
pub struct ModuleCapability {
    pub cap_type: ModuleCapType,
    /// Parâmetros específicos (ex: range de MMIO)
    pub param0: u64,
    pub param1: u64,
}

impl ModuleCapability {
    pub const fn new(cap_type: ModuleCapType) -> Self {
        Self {
            cap_type,
            param0: 0,
            param1: 0,
        }
    }
    
    pub const fn with_range(cap_type: ModuleCapType, start: u64, end: u64) -> Self {
        Self {
            cap_type,
            param0: start,
            param1: end,
        }
    }
}
```

### 4.3 `loader.rs`

```rust
//! Carregador de módulos ELF

use crate::sys::elf::{Elf64Header, Elf64Phdr, ELF_MAGIC, PhType, PF_X, PF_W, PF_R};
use crate::mm::{VirtAddr, PhysFrame, MapFlags};
use super::{ModuleError, ModuleInfo};

/// Carrega módulo da memória
pub struct ModuleLoader;

impl ModuleLoader {
    /// Valida e carrega módulo
    pub fn load(data: &[u8]) -> Result<LoadedImage, ModuleError> {
        // 1. Verificar tamanho mínimo
        if data.len() < core::mem::size_of::<Elf64Header>() {
            return Err(ModuleError::InvalidFormat);
        }
        
        // 2. Parsear header ELF
        let header = unsafe {
            &*(data.as_ptr() as *const Elf64Header)
        };
        
        if !header.is_valid() {
            return Err(ModuleError::InvalidFormat);
        }
        
        // 3. Verificar tipo (deve ser relocatable ou shared)
        if header.elf_type != 1 && header.elf_type != 3 {
            return Err(ModuleError::InvalidFormat);
        }
        
        // 4. Processar program headers
        let phoff = header.phoff as usize;
        let phnum = header.phnum as usize;
        let phentsize = header.phentsize as usize;
        
        for i in 0..phnum {
            let ph_offset = phoff + i * phentsize;
            if ph_offset + phentsize > data.len() {
                return Err(ModuleError::InvalidFormat);
            }
            
            let phdr = unsafe {
                &*(data.as_ptr().add(ph_offset) as *const Elf64Phdr)
            };
            
            if phdr.p_type == PhType::Load as u32 {
                // TODO: Mapear segmento
            }
        }
        
        Ok(LoadedImage {
            entry: VirtAddr::new(header.entry),
            base: VirtAddr::new(0), // TODO
            size: 0, // TODO
        })
    }
}

/// Imagem carregada
pub struct LoadedImage {
    pub entry: VirtAddr,
    pub base: VirtAddr,
    pub size: usize,
}
```

### 4.4 `supervisor.rs`

```rust
//! Supervisor de módulos

use crate::sync::Spinlock;
use alloc::vec::Vec;
use super::{ModuleError, ModuleInfo, ModuleAbi};
use super::capability::ModuleCapability;
use super::watchdog::HealthStatus;

/// ID único de módulo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(u32);

static NEXT_ID: crate::sync::AtomicCounter = crate::sync::AtomicCounter::new(1);

/// Estado do módulo
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModuleState {
    Loading,
    Running,
    Stopped,
    Failed,
    Banned,
}

/// Módulo carregado
pub struct LoadedModule {
    pub id: ModuleId,
    pub name: [u8; 32],
    pub state: ModuleState,
    pub caps: Vec<ModuleCapability>,
    pub fault_count: u32,
    pub abi: Option<ModuleAbi>,
}

/// Supervisor global
pub struct ModuleSupervisor {
    modules: Vec<LoadedModule>,
    max_modules: usize,
}

impl ModuleSupervisor {
    pub const fn new() -> Self {
        Self {
            modules: Vec::new(),
            max_modules: 64,
        }
    }
    
    /// Inicializa supervisor
    pub fn init(&mut self) {
        crate::kdebug!("(Supervisor) Inicializado");
    }
    
    /// Carrega módulo
    pub fn load_module(&mut self, path: &str) -> Result<ModuleId, ModuleError> {
        if self.modules.len() >= self.max_modules {
            return Err(ModuleError::LimitReached);
        }
        
        // TODO: Ler arquivo do VFS
        // TODO: Verificar assinatura
        // TODO: Carregar ELF
        // TODO: Configurar sandbox
        // TODO: Chamar init com timeout
        
        let id = ModuleId(NEXT_ID.inc() as u32);
        
        let mut name = [0u8; 32];
        let len = path.len().min(31);
        name[..len].copy_from_slice(&path.as_bytes()[..len]);
        
        self.modules.push(LoadedModule {
            id,
            name,
            state: ModuleState::Loading,
            caps: Vec::new(),
            fault_count: 0,
            abi: None,
        });
        
        crate::kinfo!("(Module) Loaded:", id.0 as u64);
        
        Ok(id)
    }
    
    /// Descarrega módulo
    pub fn unload_module(&mut self, id: ModuleId) -> Result<(), ModuleError> {
        let pos = self.modules.iter()
            .position(|m| m.id == id)
            .ok_or(ModuleError::NotFound)?;
        
        let module = &mut self.modules[pos];
        
        // Chamar cleanup
        if let Some(ref abi) = module.abi {
            if let Some(cleanup) = abi.cleanup {
                cleanup();
            }
        }
        
        self.modules.remove(pos);
        crate::kinfo!("(Module) Unloaded:", id.0 as u64);
        
        Ok(())
    }
    
    /// Lista módulos
    pub fn list_modules(&self) -> Vec<ModuleId> {
        self.modules.iter().map(|m| m.id).collect()
    }
    
    /// Reporta falha de módulo
    pub fn report_fault(&mut self, id: ModuleId) {
        if let Some(module) = self.modules.iter_mut().find(|m| m.id == id) {
            module.fault_count += 1;
            
            if module.fault_count >= 3 {
                module.state = ModuleState::Banned;
                crate::kerror!("(Module) Banned:", id.0 as u64);
            }
        }
    }
}

/// Supervisor global
pub static SUPERVISOR: Spinlock<ModuleSupervisor> = 
    Spinlock::new(ModuleSupervisor::new());
```

### 4.5 `verifier.rs`

```rust
//! Verificação de assinatura

/// Verifica assinatura de módulo
pub struct SignatureVerifier;

/// Resultado de verificação
#[derive(Debug)]
pub enum VerifyResult {
    Valid,
    InvalidSignature,
    ExpiredCertificate,
    UntrustedSigner,
    Corrupted,
}

impl SignatureVerifier {
    /// Verifica assinatura Ed25519
    pub fn verify_ed25519(
        data: &[u8],
        signature: &[u8; 64],
        public_key: &[u8; 32],
    ) -> VerifyResult {
        // TODO: Implementar verificação Ed25519
        // Por enquanto, aceitar tudo em dev
        #[cfg(debug_assertions)]
        return VerifyResult::Valid;
        
        #[cfg(not(debug_assertions))]
        return VerifyResult::InvalidSignature;
    }
}
```

### 4.6 `watchdog.rs`

```rust
//! Watchdog de módulos

use super::ModuleId;

/// Status de saúde
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unresponsive,
    Failed,
}

/// Watchdog para monitorar módulos
pub struct ModuleWatchdog {
    check_interval_ms: u64,
    timeout_ms: u64,
}

impl ModuleWatchdog {
    pub const fn new() -> Self {
        Self {
            check_interval_ms: 1000,
            timeout_ms: 5000,
        }
    }
    
    /// Verifica saúde de módulo
    pub fn check_health(&self, _id: ModuleId) -> HealthStatus {
        // TODO: Chamar health callback com timeout
        HealthStatus::Healthy
    }
}
```

---

## 5. ORDEM DE IMPLEMENTAÇÃO

1. `abi.rs` - Estruturas de ABI
2. `capability.rs` - Tipos de capability
3. `verifier.rs` - Verificação
4. `loader.rs` - Parser ELF
5. `sandbox.rs` - Isolamento
6. `supervisor.rs` - Gerenciamento
7. `watchdog.rs` - Monitoramento

---

## 6. DEPENDÊNCIAS

Pode importar de:
- `crate::sync`
- `crate::mm`
- `crate::fs` (para carregar)
- `crate::arch::x86_64::iommu`
- `crate::sys::elf`

---

## 7. CHECKLIST

- [ ] ABI_VERSION é verificado
- [ ] Assinatura verificada antes de carregar
- [ ] Init tem timeout
- [ ] Fault count leva a ban
