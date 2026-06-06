use serde::{Deserialize, Serialize};

use crate::WorldId;
use crate::grid::{ChunkCoord3, RegionCoord3};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct ChunkId {
    pub world_id: WorldId,
    pub coord: ChunkCoord3,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct RegionId {
    pub world_id: WorldId,
    pub coord: RegionCoord3,
}

impl ChunkId {
    pub fn new(world_id: WorldId, coord: ChunkCoord3) -> Self {
        Self { world_id, coord }
    }
}

impl RegionId {
    pub fn new(world_id: WorldId, coord: RegionCoord3) -> Self {
        Self { world_id, coord }
    }
}
