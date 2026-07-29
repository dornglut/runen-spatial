use crate::{SpatialEntry, SpatialKey};
use std::collections::HashMap;

pub struct SpatialIndexMapStorage<K: SpatialKey> {
    entries: HashMap<K, SpatialEntry<K>>,
}

impl<K: SpatialKey> Default for SpatialIndexMapStorage<K> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

impl<K: SpatialKey> SpatialIndexMapStorage<K> {
    pub fn get(&self, key: K) -> Option<&SpatialEntry<K>> {
        self.entries.get(&key)
    }

    pub fn insert(&mut self, entry: SpatialEntry<K>) -> Option<SpatialEntry<K>> {
        self.entries.insert(entry.key, entry)
    }

    pub fn remove(&mut self, key: K) -> Option<SpatialEntry<K>> {
        self.entries.remove(&key)
    }

    pub fn contains(&self, key: K) -> bool {
        self.entries.contains_key(&key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn values(&self) -> impl Iterator<Item = &SpatialEntry<K>> {
        self.entries.values()
    }
}
