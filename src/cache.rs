//! Resource cache, cached data, and discardable-memory interfaces.

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SkResourceCacheKey {
    pub namespace: u64,
    pub shared_id: u32,
    pub words: Vec<u32>,
}

impl SkResourceCacheKey {
    pub fn new(namespace: u64, shared_id: u32, words: Vec<u32>) -> Self {
        Self {
            namespace,
            shared_id,
            words,
        }
    }
    pub fn size(&self) -> usize {
        (self.words.len() + 2) * 4
    }
    pub fn hash(&self) -> u32 {
        let mut h = self.namespace as u32;
        for w in &self.words {
            h = crate::codec::sk_checksum_mix(h ^ *w);
        }
        h
    }
}

pub trait SkResourceCacheRec {
    fn key(&self) -> &SkResourceCacheKey;
    fn bytes_used(&self) -> usize;
    fn can_be_purged(&self) -> bool {
        true
    }
    fn post_add_install(&mut self) {}
}

pub struct SkResourceCache {
    entries: Vec<Box<dyn SkResourceCacheRec>>,
    byte_limit: usize,
    total_bytes: usize,
}

impl Default for SkResourceCache {
    fn default() -> Self {
        Self::new(32 * 1024 * 1024)
    }
}

impl SkResourceCache {
    pub fn new(byte_limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            byte_limit,
            total_bytes: 0,
        }
    }
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    pub fn total_bytes_used(&self) -> usize {
        self.total_bytes
    }
    pub fn find(&mut self, key: &SkResourceCacheKey) -> Option<&dyn SkResourceCacheRec> {
        let index = self.entries.iter().position(|e| e.key() == key)?;
        let entry = self.entries.remove(index);
        self.entries.insert(0, entry);
        Some(self.entries[0].as_ref())
    }
    pub fn add(&mut self, mut rec: Box<dyn SkResourceCacheRec>) {
        if let Some(index) = self.entries.iter().position(|e| e.key() == rec.key()) {
            if !self.entries[index].can_be_purged() {
                return;
            }
            self.total_bytes -= self.entries[index].bytes_used();
            self.entries.remove(index);
        }
        self.total_bytes += rec.bytes_used();
        rec.post_add_install();
        self.entries.insert(0, rec);
        self.purge_as_needed(false);
    }
    pub fn remove(&mut self, key: &SkResourceCacheKey) -> bool {
        if let Some(index) = self.entries.iter().position(|e| e.key() == key) {
            if !self.entries[index].can_be_purged() {
                return false;
            }
            self.total_bytes -= self.entries[index].bytes_used();
            self.entries.remove(index);
            true
        } else {
            false
        }
    }
    pub fn purge_all(&mut self) {
        self.entries.retain(|e| !e.can_be_purged());
        self.total_bytes = self.entries.iter().map(|e| e.bytes_used()).sum();
    }
    pub fn purge_as_needed(&mut self, force: bool) {
        if self.total_bytes <= self.byte_limit && !force {
            return;
        }
        let mut index = self.entries.len();
        while index > 0 && (self.total_bytes > self.byte_limit || force) {
            index -= 1;
            if self.entries[index].can_be_purged() {
                self.total_bytes -= self.entries[index].bytes_used();
                self.entries.remove(index);
            }
        }
    }
    pub fn set_total_byte_limit(&mut self, limit: usize) {
        self.byte_limit = limit;
        self.purge_as_needed(false);
    }
    pub fn purge_shared_id(&mut self, shared_id: u32) {
        if shared_id == 0 {
            return;
        }
        self.entries.retain(|e| {
            let remove = e.key().shared_id == shared_id && e.can_be_purged();
            if remove {
                self.total_bytes -= e.bytes_used();
            }
            !remove
        });
    }
}

pub struct SkCachedData {
    data: Vec<u8>,
    locked: bool,
    in_cache: bool,
    ref_count: usize,
}

impl SkCachedData {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            locked: true,
            in_cache: false,
            ref_count: 1,
        }
    }
    pub fn size(&self) -> usize {
        self.data.len()
    }
    pub fn data(&self) -> Option<&[u8]> {
        if self.locked { Some(&self.data) } else { None }
    }
    pub fn writable_data(&mut self) -> Option<&mut [u8]> {
        if self.locked {
            Some(&mut self.data)
        } else {
            None
        }
    }
    pub fn is_locked(&self) -> bool {
        self.locked
    }
    pub fn unlock(&mut self) {
        self.locked = false;
    }
    pub fn lock(&mut self) -> bool {
        self.locked = true;
        true
    }
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
    pub fn ref_(&mut self) {
        self.ref_count += 1;
    }
    pub fn unref(&mut self) -> bool {
        self.ref_count = self.ref_count.saturating_sub(1);
        self.ref_count == 0
    }
    pub fn ref_count(&self) -> usize {
        self.ref_count
    }
    pub fn set_in_cache(&mut self, in_cache: bool) {
        self.in_cache = in_cache;
    }
    pub fn in_cache(&self) -> bool {
        self.in_cache
    }
}

pub trait SkDiscardableMemory {
    fn data(&self) -> Option<&[u8]>;
    fn lock(&mut self) -> bool;
    fn unlock(&mut self);
}

pub trait SkDiscardableMemoryFactory {
    fn create(&self, bytes: usize) -> Option<Box<dyn SkDiscardableMemory>>;
}

pub fn sk_discardable_memory_create(_bytes: usize) -> Option<Box<dyn SkDiscardableMemory>> {
    None
}
