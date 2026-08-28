use crate::grid::{ChunkCoord3, ChunkId, RegionCoord3, RegionId};
use crate::{FrameLocalPosition, SpatialMathError, WorldFrame, WorldId, WorldPosition};

#[derive(Debug, Clone, PartialEq)]
pub struct GridPartitionConfig {
    chunk_edge_meters: f64,
    region_chunk_dims: [u32; 3],
}

impl Default for GridPartitionConfig {
    fn default() -> Self {
        Self::try_new(32.0, [8, 8, 8]).expect("default partition configuration is valid")
    }
}

impl GridPartitionConfig {
    pub fn try_new(
        chunk_edge_meters: f64,
        region_chunk_dims: [u32; 3],
    ) -> Result<Self, SpatialMathError> {
        if !chunk_edge_meters.is_finite() {
            return Err(SpatialMathError::NonFiniteValue {
                field: "chunk_edge_meters",
            });
        }
        if chunk_edge_meters <= 0.0 {
            return Err(SpatialMathError::NonPositiveValue {
                field: "chunk_edge_meters",
            });
        }
        for (axis, dimension) in region_chunk_dims.iter().enumerate() {
            if *dimension == 0 {
                return Err(SpatialMathError::ZeroDimension { axis: axis as u8 });
            }
        }
        Ok(Self {
            chunk_edge_meters,
            region_chunk_dims,
        })
    }

    pub fn chunk_edge_meters(&self) -> f64 {
        self.chunk_edge_meters
    }

    pub fn region_chunk_dims(&self) -> [u32; 3] {
        self.region_chunk_dims
    }

    pub fn chunk_coord_from_world_position(
        &self,
        position: WorldPosition,
    ) -> Result<ChunkCoord3, SpatialMathError> {
        let meters = position.meters();
        Ok(ChunkCoord3 {
            x: coordinate_from_meters(meters[0], self.chunk_edge_meters, "chunk x")?,
            y: coordinate_from_meters(meters[1], self.chunk_edge_meters, "chunk y")?,
            z: coordinate_from_meters(meters[2], self.chunk_edge_meters, "chunk z")?,
        })
    }

    pub fn chunk_coord_from_frame_local(
        &self,
        frame: WorldFrame,
        position: FrameLocalPosition,
    ) -> Result<ChunkCoord3, SpatialMathError> {
        self.chunk_coord_from_world_position(frame.to_global(position)?)
    }

    pub fn chunk_id_from_world_position(
        &self,
        position: WorldPosition,
    ) -> Result<ChunkId, SpatialMathError> {
        Ok(ChunkId::new(
            position.world_id(),
            self.chunk_coord_from_world_position(position)?,
        ))
    }

    pub fn chunk_id_from_frame_local(
        &self,
        frame: WorldFrame,
        position: FrameLocalPosition,
    ) -> Result<ChunkId, SpatialMathError> {
        self.chunk_id_from_world_position(frame.to_global(position)?)
    }

    pub fn region_coord_from_chunk_coord(&self, chunk: ChunkCoord3) -> RegionCoord3 {
        RegionCoord3 {
            x: chunk.x.div_euclid(i64::from(self.region_chunk_dims[0])),
            y: chunk.y.div_euclid(i64::from(self.region_chunk_dims[1])),
            z: chunk.z.div_euclid(i64::from(self.region_chunk_dims[2])),
        }
    }

    pub fn region_id_from_chunk_id(&self, chunk_id: ChunkId) -> RegionId {
        RegionId::new(
            chunk_id.world_id,
            self.region_coord_from_chunk_coord(chunk_id.coord),
        )
    }

    pub fn chunk_origin_world_position(
        &self,
        world_id: WorldId,
        chunk: ChunkCoord3,
    ) -> Result<WorldPosition, SpatialMathError> {
        let meters = [
            (chunk.x as f64) * self.chunk_edge_meters,
            (chunk.y as f64) * self.chunk_edge_meters,
            (chunk.z as f64) * self.chunk_edge_meters,
        ];
        if meters.iter().any(|value| !value.is_finite()) {
            return Err(SpatialMathError::ArithmeticOverflow {
                operation: "chunk origin",
            });
        }
        let position = WorldPosition::try_new(world_id, meters)?;
        if self.chunk_coord_from_world_position(position)? != chunk {
            return Err(SpatialMathError::PrecisionLoss {
                operation: "chunk origin",
            });
        }
        Ok(position)
    }
}

pub(crate) fn coordinate_from_meters(
    meters: f64,
    edge: f64,
    operation: &'static str,
) -> Result<i64, SpatialMathError> {
    if !meters.is_finite() {
        return Err(SpatialMathError::NonFiniteValue { field: operation });
    }
    let value = (meters / edge).floor();
    if !value.is_finite()
        || !(-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&value)
    {
        return Err(SpatialMathError::CoordinateOutOfRange { operation });
    }
    Ok(value as i64)
}
