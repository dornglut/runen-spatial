use crate::SpatialKey;
use runen_spatial::SpatialAabb3;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AabbQuery {
    pub bounds: SpatialAabb3,
}

impl AabbQuery {
    pub fn new(bounds: SpatialAabb3) -> Self {
        Self { bounds }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult<K: SpatialKey> {
    pub keys: Vec<K>,
}

impl<K: SpatialKey> QueryResult<K> {
    pub fn new(keys: Vec<K>) -> Self {
        Self { keys }
    }

    pub fn into_keys(self) -> Vec<K> {
        self.keys
    }
}
