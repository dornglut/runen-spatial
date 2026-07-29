use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct ChunkCoord3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct RegionCoord3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
