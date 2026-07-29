use runen_spatial::ChunkCoord3;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ChunkPriority {
    pub rank: u32,
    pub distance_squared: i128,
}

impl ChunkPriority {
    pub fn new(rank: u32, distance_squared: i128) -> Self {
        Self {
            rank,
            distance_squared,
        }
    }
}

pub(crate) fn distance_squared(a: ChunkCoord3, b: ChunkCoord3) -> i128 {
    let dx = i128::from(a.x) - i128::from(b.x);
    let dy = i128::from(a.y) - i128::from(b.y);
    let dz = i128::from(a.z) - i128::from(b.z);
    dx * dx + dy * dy + dz * dz
}
