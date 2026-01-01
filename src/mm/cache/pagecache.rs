//! # Page Cache

use crate::mm::{PhysAddr, PhysFrame};
use crate::sync::Spinlock;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub type FileId = u64;
pub type PageIndex = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CacheKey {
    pub file_id: FileId,
    pub page_index: PageIndex,
}

impl CacheKey {
    pub const fn new(file_id: FileId, page_index: PageIndex) -> Self {
        Self {
            file_id,
            page_index,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageState {
    Clean,
    Dirty,
    Reading,
    Writing,
    Invalid,
}

pub struct CachedPage {
    pub key: CacheKey,
    pub frame: PhysAddr,
    pub state: PageState,
    pub use_count: AtomicU32,
    pub last_access: AtomicU64,
    pub pinned: AtomicBool,
    pub map_count: AtomicU32,
}

impl CachedPage {
    pub fn new(key: CacheKey, frame: PhysAddr) -> Self {
        Self {
            key,
            frame,
            state: PageState::Clean,
            use_count: AtomicU32::new(1),
            last_access: AtomicU64::new(0),
            pinned: AtomicBool::new(false),
            map_count: AtomicU32::new(0),
        }
    }

    pub fn touch(&self, timestamp: u64) {
        self.use_count.fetch_add(1, Ordering::Relaxed);
        self.last_access.store(timestamp, Ordering::Relaxed);
    }

    pub fn can_evict(&self) -> bool {
        !self.pinned.load(Ordering::Acquire)
            && self.map_count.load(Ordering::Acquire) == 0
            && self.state != PageState::Writing
            && self.state != PageState::Reading
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PageCacheStats {
    pub cached_pages: u64,
    pub hits: u64,
    pub misses: u64,
    pub writebacks: u64,
    pub evictions: u64,
    pub dirty_pages: u64,
}

pub struct PageCache {
    pages: BTreeMap<CacheKey, CachedPage>,
    lru_list: Vec<CacheKey>,
    max_pages: usize,
    stats: PageCacheStats,
    current_time: u64,
}

impl PageCache {
    pub fn new(max_pages: usize) -> Self {
        Self {
            pages: BTreeMap::new(),
            lru_list: Vec::with_capacity(max_pages),
            max_pages,
            stats: PageCacheStats::default(),
            current_time: 0,
        }
    }

    pub fn lookup(&mut self, file_id: FileId, page_index: PageIndex) -> Option<&CachedPage> {
        let key = CacheKey::new(file_id, page_index);
        self.current_time += 1;

        if let Some(page) = self.pages.get(&key) {
            page.touch(self.current_time);
            self.stats.hits += 1;
            Some(page)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, file_id: FileId, page_index: PageIndex, frame: PhysAddr) -> bool {
        let key = CacheKey::new(file_id, page_index);

        if self.pages.contains_key(&key) {
            return false;
        }

        if self.pages.len() >= self.max_pages {
            if !self.try_evict_one() {
                return false;
            }
        }

        let page = CachedPage::new(key, frame);
        self.pages.insert(key, page);
        self.lru_list.push(key);
        self.stats.cached_pages = self.pages.len() as u64;
        true
    }

    pub fn remove(&mut self, file_id: FileId, page_index: PageIndex) -> Option<CachedPage> {
        let key = CacheKey::new(file_id, page_index);
        if let Some(pos) = self.lru_list.iter().position(|k| *k == key) {
            self.lru_list.remove(pos);
        }
        let page = self.pages.remove(&key);
        if page.is_some() {
            self.stats.cached_pages = self.pages.len() as u64;
        }
        page
    }

    fn try_evict_one(&mut self) -> bool {
        let mut victim_idx = None;
        let mut min_access = u64::MAX;

        for (idx, key) in self.lru_list.iter().enumerate() {
            if let Some(page) = self.pages.get(key) {
                if page.can_evict() {
                    let access = page.last_access.load(Ordering::Relaxed);
                    if access < min_access {
                        min_access = access;
                        victim_idx = Some(idx);
                    }
                }
            }
        }

        if let Some(idx) = victim_idx {
            let key = self.lru_list.remove(idx);
            if let Some(page) = self.pages.remove(&key) {
                if page.state == PageState::Dirty {
                    self.stats.writebacks += 1;
                }
                crate::mm::pmm::FRAME_ALLOCATOR
                    .lock()
                    .deallocate_frame(page.frame);
                self.stats.evictions += 1;
                self.stats.cached_pages = self.pages.len() as u64;
                return true;
            }
        }
        false
    }

    pub fn stats(&self) -> PageCacheStats {
        self.stats
    }
    pub fn len(&self) -> usize {
        self.pages.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }
}

const DEFAULT_CACHE_PAGES: usize = 65536;
static PAGE_CACHE: Spinlock<Option<PageCache>> = Spinlock::new(None);

pub fn init(max_pages: usize) {
    let mut cache = PAGE_CACHE.lock();
    *cache = Some(PageCache::new(max_pages));
}

pub fn init_default() {
    init(DEFAULT_CACHE_PAGES);
}

pub fn with_cache<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut PageCache) -> R,
{
    let mut guard = PAGE_CACHE.lock();
    guard.as_mut().map(f)
}

pub fn lookup(file_id: FileId, page_index: PageIndex) -> Option<PhysAddr> {
    with_cache(|cache| cache.lookup(file_id, page_index).map(|p| p.frame)).flatten()
}

pub fn insert(file_id: FileId, page_index: PageIndex, frame: PhysAddr) -> bool {
    with_cache(|cache| cache.insert(file_id, page_index, frame)).unwrap_or(false)
}

pub fn stats() -> PageCacheStats {
    with_cache(|cache| cache.stats()).unwrap_or_default()
}
