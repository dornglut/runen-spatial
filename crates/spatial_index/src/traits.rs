use crate::{AabbQuery, QueryResult, SpatialEntry, SpatialIndexError, SpatialKey};
use spatial::SpatialAabb3;

pub trait SpatialIndex<K: SpatialKey> {
    fn contains(&self, key: K) -> bool;

    fn bounds(&self, key: K) -> Option<SpatialAabb3>;

    fn query_aabb(&self, query: AabbQuery) -> Result<QueryResult<K>, SpatialIndexError>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait MutableSpatialIndex<K: SpatialKey>: SpatialIndex<K> {
    fn insert(&mut self, entry: SpatialEntry<K>) -> Result<(), SpatialIndexError>;

    fn update(&mut self, entry: SpatialEntry<K>) -> Result<(), SpatialIndexError>;

    fn upsert(&mut self, entry: SpatialEntry<K>) -> Result<(), SpatialIndexError>;

    fn remove(&mut self, key: K) -> bool;

    fn clear(&mut self);
}
