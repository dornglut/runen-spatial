use crate::SpatialKey;
use spatial::SpatialAabb3;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SpatialEntry<K: SpatialKey> {
    pub key: K,
    pub bounds: SpatialAabb3,
}

impl<K: SpatialKey> SpatialEntry<K> {
    pub fn new(key: K, bounds: SpatialAabb3) -> Self {
        Self { key, bounds }
    }
}
