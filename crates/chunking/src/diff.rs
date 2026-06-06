use spatial::ChunkCoord3;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkSetDiff {
    pub entered: Vec<ChunkCoord3>,
    pub exited: Vec<ChunkCoord3>,
}

impl ChunkSetDiff {
    pub fn is_empty(&self) -> bool {
        self.entered.is_empty() && self.exited.is_empty()
    }

    pub fn clear(&mut self) {
        self.entered.clear();
        self.exited.clear();
    }
}
