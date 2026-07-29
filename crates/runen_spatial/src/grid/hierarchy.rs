use serde::{Deserialize, Serialize};

use crate::WorldId;
use crate::grid::ChunkCoord3;

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct GridLevel(pub u8);

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct HierarchicalGridConfig {
    pub base_chunk_edge_meters: f32,
    pub level_count: u8,
    pub level_scale_factor: u32,
}

impl Default for HierarchicalGridConfig {
    fn default() -> Self {
        Self {
            base_chunk_edge_meters: 32.0,
            level_count: 1,
            level_scale_factor: 2,
        }
    }
}

impl HierarchicalGridConfig {
    pub fn cell_edge_meters_for_level(&self, level: GridLevel) -> f32 {
        let factor = self.level_scale_factor.max(1) as f32;
        self.base_chunk_edge_meters.max(1.0) * factor.powi(level.0 as i32)
    }

    pub fn parent_level(&self, level: GridLevel) -> Option<GridLevel> {
        (level.0 > 0).then_some(GridLevel(level.0 - 1))
    }

    pub fn child_level(&self, level: GridLevel) -> Option<GridLevel> {
        (level.0 + 1 < self.level_count.max(1)).then_some(GridLevel(level.0 + 1))
    }

    pub fn parent_coord(&self, coord: ChunkCoord3) -> ChunkCoord3 {
        let scale = self.level_scale_factor.max(1) as i32;

        ChunkCoord3 {
            x: div_floor(coord.x, scale),
            y: div_floor(coord.y, scale),
            z: div_floor(coord.z, scale),
        }
    }

    pub fn first_child_coord(&self, coord: ChunkCoord3) -> ChunkCoord3 {
        let scale = self.level_scale_factor.max(1) as i32;

        ChunkCoord3 {
            x: coord.x * scale,
            y: coord.y * scale,
            z: coord.z * scale,
        }
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct HierarchicalChunkId {
    pub world_id: WorldId,
    pub level: GridLevel,
    pub coord: ChunkCoord3,
}

impl HierarchicalChunkId {
    pub fn new(world_id: WorldId, level: GridLevel, coord: ChunkCoord3) -> Self {
        Self {
            world_id,
            level,
            coord,
        }
    }

    pub fn parent(&self, config: &HierarchicalGridConfig) -> Option<Self> {
        let parent_level = config.parent_level(self.level)?;
        Some(Self {
            world_id: self.world_id,
            level: parent_level,
            coord: config.parent_coord(self.coord),
        })
    }
}

fn div_floor(value: i32, divisor: i32) -> i32 {
    let mut out = value / divisor;
    let remainder = value % divisor;
    if remainder != 0 && ((remainder < 0) != (divisor < 0)) {
        out -= 1;
    }
    out
}
