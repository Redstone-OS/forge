//! Fast Userspace Mutex

use crate::sync::Spinlock;
use crate::sched::wait::WaitQueue;
use crate::mm::VirtAddr;
use alloc::collections::BTreeMap;

/// Tabela global de futexes
static FUTEX_TABLE: Spinlock<BTreeMap<u64, WaitQueue>> = 
    Spinlock::new(BTreeMap::new());

/// Futex - primitiva de sincronização userspace
pub struct Futex;

impl Futex {
    /// Wait: dorme se *addr == expected
    pub fn wait(addr: VirtAddr, expected: u32) -> Result<(), FutexError> {
        // Ler valor atual
        let current = unsafe { *(addr.as_ptr::<u32>()) };
        
        if current != expected {
            return Err(FutexError::WouldBlock);
        }
        
        // Adicionar à wait queue
        let mut table = FUTEX_TABLE.lock();
        let queue = table.entry(addr.as_u64())
            .or_insert_with(WaitQueue::new);
        
        // Dormir
        queue.wait();
        
        Ok(())
    }
    
    /// Wake: acorda até N threads esperando em addr
    pub fn wake(addr: VirtAddr, count: u32) -> u32 {
        let table = FUTEX_TABLE.lock();
        
        if let Some(queue) = table.get(&addr.as_u64()) {
            let mut woken = 0;
            for _ in 0..count {
                queue.wake_one();
                woken += 1;
            }
            woken
        } else {
            0
        }
    }
}

#[derive(Debug)]
pub enum FutexError {
    WouldBlock,
    InvalidAddress,
}
