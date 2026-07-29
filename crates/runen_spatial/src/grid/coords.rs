use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct ChunkCoord3 {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct RegionCoord3 {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl ChunkCoord3 {
    pub fn checked_add(self, offset: Self) -> Option<Self> {
        Some(Self {
            x: self.x.checked_add(offset.x)?,
            y: self.y.checked_add(offset.y)?,
            z: self.z.checked_add(offset.z)?,
        })
    }
    pub fn checked_offset(self, x: i64, y: i64, z: i64) -> Option<Self> {
        self.checked_add(Self { x, y, z })
    }
    pub fn checked_mul(self, scale: i64) -> Option<Self> {
        Some(Self {
            x: self.x.checked_mul(scale)?,
            y: self.y.checked_mul(scale)?,
            z: self.z.checked_mul(scale)?,
        })
    }
}
