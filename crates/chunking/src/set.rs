use spatial::ChunkCoord3;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkSet {
    pub chunks: BTreeSet<ChunkCoord3>,
}

impl ChunkSet {
    pub fn contains(&self, chunk: &ChunkCoord3) -> bool {
        self.chunks.contains(chunk)
    }

    pub fn insert(&mut self, chunk: ChunkCoord3) -> bool {
        self.chunks.insert(chunk)
    }

    pub fn remove(&mut self, chunk: &ChunkCoord3) -> bool {
        self.chunks.remove(chunk)
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &ChunkCoord3> {
        self.chunks.iter()
    }
}
