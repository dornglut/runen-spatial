use serde::{Deserialize, Deserializer, Serialize};

use crate::grid::ChunkCoord3;
use crate::{SpatialMathError, WorldId};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct GridLevel(pub u8);

#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub struct HierarchicalGridConfig {
    base_chunk_edge_meters: f64,
    level_count: u8,
    level_scale_factor: u32,
}

#[derive(Deserialize)]
struct RawHierarchicalGridConfig {
    base_chunk_edge_meters: f64,
    level_count: u8,
    level_scale_factor: u32,
}

impl<'de> Deserialize<'de> for HierarchicalGridConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawHierarchicalGridConfig::deserialize(deserializer)?;
        Self::try_new(
            raw.base_chunk_edge_meters,
            raw.level_count,
            raw.level_scale_factor,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl Default for HierarchicalGridConfig {
    fn default() -> Self {
        Self::try_new(32.0, 1, 2).expect("default hierarchy configuration is valid")
    }
}

impl HierarchicalGridConfig {
    pub fn try_new(
        base_chunk_edge_meters: f64,
        level_count: u8,
        level_scale_factor: u32,
    ) -> Result<Self, SpatialMathError> {
        if !base_chunk_edge_meters.is_finite() {
            return Err(SpatialMathError::NonFiniteValue {
                field: "base_chunk_edge_meters",
            });
        }
        if base_chunk_edge_meters <= 0.0 {
            return Err(SpatialMathError::NonPositiveValue {
                field: "base_chunk_edge_meters",
            });
        }
        if level_count == 0 {
            return Err(SpatialMathError::LevelCountZero);
        }
        if level_scale_factor < 2 {
            return Err(SpatialMathError::ScaleFactorTooSmall {
                scale_factor: level_scale_factor,
            });
        }
        Ok(Self {
            base_chunk_edge_meters,
            level_count,
            level_scale_factor,
        })
    }

    pub fn base_chunk_edge_meters(&self) -> f64 {
        self.base_chunk_edge_meters
    }
    pub fn level_count(&self) -> u8 {
        self.level_count
    }
    pub fn level_scale_factor(&self) -> u32 {
        self.level_scale_factor
    }

    pub fn validate_level(&self, level: GridLevel) -> Result<(), SpatialMathError> {
        if level.0 >= self.level_count {
            Err(SpatialMathError::LevelOutOfRange {
                level: level.0,
                level_count: self.level_count,
            })
        } else {
            Ok(())
        }
    }

    pub fn cell_edge_meters_for_level(&self, level: GridLevel) -> Result<f64, SpatialMathError> {
        self.validate_level(level)?;
        let edge =
            self.base_chunk_edge_meters * (self.level_scale_factor as f64).powi(i32::from(level.0));
        if !edge.is_finite() {
            return Err(SpatialMathError::ArithmeticOverflow {
                operation: "hierarchical cell edge",
            });
        }
        Ok(edge)
    }

    pub fn parent_level(&self, level: GridLevel) -> Result<Option<GridLevel>, SpatialMathError> {
        self.validate_level(level)?;
        if level.0 + 1 < self.level_count {
            Ok(Some(GridLevel(level.0 + 1)))
        } else {
            Ok(None)
        }
    }

    pub fn child_level(&self, level: GridLevel) -> Result<Option<GridLevel>, SpatialMathError> {
        self.validate_level(level)?;
        if level.0 > 0 {
            Ok(Some(GridLevel(level.0 - 1)))
        } else {
            Ok(None)
        }
    }

    pub fn parent_coord(
        &self,
        level: GridLevel,
        coord: ChunkCoord3,
    ) -> Result<ChunkCoord3, SpatialMathError> {
        if self.parent_level(level)?.is_none() {
            return Err(SpatialMathError::LevelOutOfRange {
                level: level.0,
                level_count: self.level_count,
            });
        }
        let scale = i64::from(self.level_scale_factor);
        Ok(ChunkCoord3 {
            x: coord.x.div_euclid(scale),
            y: coord.y.div_euclid(scale),
            z: coord.z.div_euclid(scale),
        })
    }

    pub fn first_child_coord(
        &self,
        level: GridLevel,
        coord: ChunkCoord3,
    ) -> Result<ChunkCoord3, SpatialMathError> {
        if self.child_level(level)?.is_none() {
            return Err(SpatialMathError::LevelOutOfRange {
                level: level.0,
                level_count: self.level_count,
            });
        }
        coord.checked_mul(i64::from(self.level_scale_factor)).ok_or(
            SpatialMathError::ArithmeticOverflow {
                operation: "first child coordinate",
            },
        )
    }

    pub fn child_coord_bounds(
        &self,
        level: GridLevel,
        coord: ChunkCoord3,
    ) -> Result<(ChunkCoord3, ChunkCoord3), SpatialMathError> {
        let first = self.first_child_coord(level, coord)?;
        let offset = i64::from(self.level_scale_factor) - 1;
        let last = first.checked_offset(offset, offset, offset).ok_or(
            SpatialMathError::ArithmeticOverflow {
                operation: "last child coordinate",
            },
        )?;
        Ok((first, last))
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

    pub fn parent(
        &self,
        config: &HierarchicalGridConfig,
    ) -> Result<Option<Self>, SpatialMathError> {
        match config.parent_level(self.level)? {
            Some(level) => Ok(Some(Self {
                world_id: self.world_id,
                level,
                coord: config.parent_coord(self.level, self.coord)?,
            })),
            None => Ok(None),
        }
    }
}
