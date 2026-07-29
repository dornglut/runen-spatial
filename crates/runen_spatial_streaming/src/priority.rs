use runen_spatial::ChunkCoord3;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ChunkPriority {
    pub rank: u32,
    pub distance_squared: i64,
}

impl ChunkPriority {
    pub fn new(rank: u32, distance_squared: i64) -> Self {
        Self {
            rank,
            distance_squared,
        }
    }
}

pub(crate) fn distance_squared(a: ChunkCoord3, b: ChunkCoord3) -> i64 {
    let dx = i64::from(a.x) - i64::from(b.x);
    let dy = i64::from(a.y) - i64::from(b.y);
    let dz = i64::from(a.z) - i64::from(b.z);
    dx * dx + dy * dy + dz * dz
}
