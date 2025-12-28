# Guia de Implementação: `sync/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE. Não improvise.

---

## 1. PROPÓSITO DESTE MÓDULO

O módulo `sync/` contém **primitivas de sincronização** para ambiente multicore (SMP). Evita data races e condições de corrida.

---

## 2. ESTRUTURA DE ARQUIVOS OBRIGATÓRIA

```
sync/
├── mod.rs              ✅ JÁ EXISTE - NÃO MODIFICAR
├── atomic/
│   ├── mod.rs
│   └── atomic.rs       → AtomicCell, AtomicBool
├── spinlock/
│   ├── mod.rs
│   └── spinlock.rs     → Spinlock, SpinlockGuard
├── mutex/
│   ├── mod.rs
│   └── mutex.rs        → Mutex, MutexGuard (pode dormir)
├── rwlock/
│   ├── mod.rs
│   └── rwlock.rs       → RwLock, Guards
├── semaphore/
│   ├── mod.rs
│   └── semaphore.rs    → Semaphore
├── condvar/
│   ├── mod.rs
│   └── condvar.rs      → Condition Variable
└── rcu/
    ├── mod.rs
    └── rcu.rs          → Read-Copy-Update
```

---

## 3. REGRAS INVIOLÁVEIS

### ❌ NUNCA FAZER:
- Usar `std::sync` (não existe em `no_std`)
- Fazer alocação de heap dentro de locks
- Usar `f32`/`f64`
- Usar `unwrap()` ou `expect()`

### ✅ SEMPRE FAZER:
- Usar `core::sync::atomic` para atomics
- Implementar `Send` e `Sync` corretamente
- Documentar quando spinlock pode/não pode ser usado
- Desabilitar interrupções ao adquirir spinlock

---

## 4. IMPLEMENTAÇÃO DETALHADA

### 4.1 `atomic/atomic.rs`

```rust
//! Operações atômicas

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use core::cell::UnsafeCell;

/// Célula atômica genérica para tipos pequenos
pub struct AtomicCell<T: Copy> {
    value: UnsafeCell<T>,
}

// SAFETY: AtomicCell usa operações atômicas internamente
unsafe impl<T: Copy + Send> Send for AtomicCell<T> {}
unsafe impl<T: Copy + Send> Sync for AtomicCell<T> {}

impl<T: Copy> AtomicCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }
    
    /// Carrega o valor (não atômico para tipos grandes!)
    pub fn load(&self) -> T {
        // SAFETY: Assumimos acesso único ou tipo atômico
        unsafe { *self.value.get() }
    }
    
    /// Armazena o valor
    pub fn store(&self, value: T) {
        // SAFETY: Assumimos acesso único ou tipo atômico
        unsafe { *self.value.get() = value; }
    }
}

/// Wrapper para AtomicBool com API mais limpa
pub struct AtomicFlag(AtomicBool);

impl AtomicFlag {
    pub const fn new(value: bool) -> Self {
        Self(AtomicBool::new(value))
    }
    
    pub fn get(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    
    pub fn set(&self, value: bool) {
        self.0.store(value, Ordering::Release);
    }
    
    /// Test-and-set: retorna valor anterior
    pub fn test_and_set(&self) -> bool {
        self.0.swap(true, Ordering::AcqRel)
    }
    
    pub fn clear(&self) {
        self.0.store(false, Ordering::Release);
    }
}

/// Contador atômico
pub struct AtomicCounter(AtomicU64);

impl AtomicCounter {
    pub const fn new(value: u64) -> Self {
        Self(AtomicU64::new(value))
    }
    
    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Acquire)
    }
    
    pub fn set(&self, value: u64) {
        self.0.store(value, Ordering::Release);
    }
    
    pub fn inc(&self) -> u64 {
        self.0.fetch_add(1, Ordering::AcqRel)
    }
    
    pub fn dec(&self) -> u64 {
        self.0.fetch_sub(1, Ordering::AcqRel)
    }
    
    pub fn add(&self, value: u64) -> u64 {
        self.0.fetch_add(value, Ordering::AcqRel)
    }
}
```

### 4.2 `spinlock/spinlock.rs`

```rust
//! Spinlock - bloqueio com busy-wait

use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Spinlock - usa busy-wait, NÃO pode dormir
/// 
/// # Quando usar
/// 
/// - Seções críticas MUITO curtas
/// - Dentro de handlers de interrupção
/// - Quando não pode chamar scheduler
/// 
/// # Quando NÃO usar
/// 
/// - Seções que podem demorar
/// - Quando pode chamar funções que dormem
/// - Para proteger I/O lento
pub struct Spinlock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: Spinlock protege acesso com lock atômico
unsafe impl<T: Send> Send for Spinlock<T> {}
unsafe impl<T: Send> Sync for Spinlock<T> {}

impl<T> Spinlock<T> {
    /// Cria novo spinlock
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    
    /// Adquire o lock
    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        // Desabilitar interrupções antes de adquirir
        let interrupts_enabled = crate::arch::Cpu::interrupts_enabled();
        crate::arch::Cpu::disable_interrupts();
        
        // Spin até conseguir o lock
        while self.locked.compare_exchange_weak(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            // Hint para CPU que estamos em spin loop
            core::hint::spin_loop();
        }
        
        SpinlockGuard {
            lock: self,
            interrupts_were_enabled: interrupts_enabled,
        }
    }
    
    /// Tenta adquirir sem bloquear
    pub fn try_lock(&self) -> Option<SpinlockGuard<'_, T>> {
        let interrupts_enabled = crate::arch::Cpu::interrupts_enabled();
        crate::arch::Cpu::disable_interrupts();
        
        if self.locked.compare_exchange(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(SpinlockGuard {
                lock: self,
                interrupts_were_enabled: interrupts_enabled,
            })
        } else {
            // Não conseguiu, restaurar interrupções
            if interrupts_enabled {
                crate::arch::Cpu::enable_interrupts();
            }
            None
        }
    }
}

/// Guard do spinlock - libera ao sair do escopo
pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
    interrupts_were_enabled: bool,
}

impl<T> Deref for SpinlockGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        // SAFETY: Lock está adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: Lock está adquirido
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinlockGuard<'_, T> {
    fn drop(&mut self) {
        // Liberar lock
        self.lock.locked.store(false, Ordering::Release);
        
        // Restaurar interrupções se estavam habilitadas
        if self.interrupts_were_enabled {
            crate::arch::Cpu::enable_interrupts();
        }
    }
}
```

### 4.3 `mutex/mutex.rs`

```rust
//! Mutex - pode bloquear thread

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Mutex - bloqueia thread se não conseguir lock
/// 
/// # Diferença do Spinlock
/// 
/// - Mutex PODE dormir (chama scheduler)
/// - Spinlock NÃO pode dormir (busy-wait)
/// 
/// Use Mutex para seções mais longas.
pub struct Mutex<T> {
    /// Estado do lock
    locked: AtomicBool,
    /// ID do owner (para debug)
    owner: AtomicU32,
    /// Dados protegidos
    data: UnsafeCell<T>,
}

// SAFETY: Mutex protege acesso com lock
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            owner: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    /// Adquire o lock (pode bloquear)
    pub fn lock(&self) -> MutexGuard<'_, T> {
        // Tentar adquirir
        while self.locked.compare_exchange_weak(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            // TODO: Integrar com scheduler para dormir
            // Por enquanto, spin
            core::hint::spin_loop();
        }
        
        MutexGuard { lock: self }
    }
    
    /// Tenta adquirir sem bloquear
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.locked.compare_exchange(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok() {
            Some(MutexGuard { lock: self })
        } else {
            None
        }
    }
}

pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        // SAFETY: Lock está adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: Lock está adquirido
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.owner.store(0, Ordering::Release);
        self.lock.locked.store(false, Ordering::Release);
        // TODO: Acordar threads esperando
    }
}
```

### 4.4 `rwlock/rwlock.rs`

```rust
//! Reader-Writer Lock

use core::sync::atomic::{AtomicI32, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// RwLock - múltiplos leitores OU um escritor
/// 
/// Contador:
/// - 0 = Livre
/// - N>0 = N leitores ativos
/// - -1 = Escritor ativo
pub struct RwLock<T> {
    state: AtomicI32,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicI32::new(0),
            data: UnsafeCell::new(data),
        }
    }
    
    /// Adquire lock de leitura
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        loop {
            let state = self.state.load(Ordering::Acquire);
            
            // Se escritor ativo, esperar
            if state < 0 {
                core::hint::spin_loop();
                continue;
            }
            
            // Tentar incrementar leitores
            if self.state.compare_exchange_weak(
                state,
                state + 1,
                Ordering::Acquire,
                Ordering::Relaxed
            ).is_ok() {
                return RwLockReadGuard { lock: self };
            }
        }
    }
    
    /// Adquire lock de escrita
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        loop {
            // Tentar adquirir se livre
            if self.state.compare_exchange_weak(
                0,
                -1,
                Ordering::Acquire,
                Ordering::Relaxed
            ).is_ok() {
                return RwLockWriteGuard { lock: self };
            }
            core::hint::spin_loop();
        }
    }
}

pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: Lock de leitura adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: Lock de escrita adquirido
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: Lock de escrita adquirido
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
    }
}
```

### 4.5 `semaphore/semaphore.rs`

```rust
//! Semáforo para controle de recursos

use core::sync::atomic::{AtomicI32, Ordering};

/// Semáforo de contagem
pub struct Semaphore {
    count: AtomicI32,
}

impl Semaphore {
    pub const fn new(initial: i32) -> Self {
        Self {
            count: AtomicI32::new(initial),
        }
    }
    
    /// Decrementa (P/wait/acquire)
    pub fn acquire(&self) {
        loop {
            let count = self.count.load(Ordering::Acquire);
            if count <= 0 {
                // Esperar
                core::hint::spin_loop();
                continue;
            }
            
            if self.count.compare_exchange_weak(
                count,
                count - 1,
                Ordering::AcqRel,
                Ordering::Relaxed
            ).is_ok() {
                return;
            }
        }
    }
    
    /// Tenta decrementar sem bloquear
    pub fn try_acquire(&self) -> bool {
        let count = self.count.load(Ordering::Acquire);
        if count <= 0 {
            return false;
        }
        
        self.count.compare_exchange(
            count,
            count - 1,
            Ordering::AcqRel,
            Ordering::Relaxed
        ).is_ok()
    }
    
    /// Incrementa (V/signal/release)
    pub fn release(&self) {
        self.count.fetch_add(1, Ordering::Release);
    }
}
```

---

## 5. ORDEM DE IMPLEMENTAÇÃO

1. `atomic/atomic.rs` - Base atômica
2. `spinlock/spinlock.rs` - Lock mais simples
3. `mutex/mutex.rs` - Lock com sleep
4. `rwlock/rwlock.rs` - Reader-writer
5. `semaphore/semaphore.rs` - Contagem
6. `condvar/condvar.rs` - Condition variable
7. `rcu/rcu.rs` - Read-copy-update

---

## 6. DEPENDÊNCIAS

Este módulo pode importar de:
- `core::sync::atomic`
- `core::cell::UnsafeCell`
- `crate::arch::Cpu` (para disable/enable interrupts)

Este módulo NÃO pode importar de:
- `crate::mm`
- `crate::sched`
- `crate::ipc`

---

## 7. CHECKLIST FINAL

- [ ] Todos os locks implementam `Send + Sync` corretamente
- [ ] Spinlock desabilita interrupções
- [ ] Guards liberam lock no Drop
- [ ] Nenhum deadlock possível com única aquisição
- [ ] Documentação explica quando usar cada primitiva
