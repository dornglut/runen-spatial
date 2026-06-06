use crate::grid::{ChunkCoord3, ChunkId, RegionCoord3, RegionId};
use crate::{WorldId, WorldLocalPosition};

#[derive(Debug, Clone, PartialEq)]
pub struct GridPartitionConfig {
    pub chunk_edge_meters: f32,
    pub region_chunk_dims: [u32; 3],
    pub fixed_point_scale: i32,
}

impl Default for GridPartitionConfig {
    fn default() -> Self {
        Self {
            chunk_edge_meters: 32.0,
            region_chunk_dims: [8, 8, 8],
            fixed_point_scale: 1024,
        }
    }
}

impl GridPartitionConfig {
    pub fn quantization_scale(&self) -> i32 {
        self.fixed_point_scale.max(1)
    }

    pub fn chunk_coord_from_world_local_position(
        &self,
        position: WorldLocalPosition,
    ) -> ChunkCoord3 {
        let edge = self.chunk_edge_meters.max(1.0);

        ChunkCoord3 {
            x: (position.meters[0] / edge).floor() as i32,
            y: (position.meters[1] / edge).floor() as i32,
            z: (position.meters[2] / edge).floor() as i32,
        }
    }

    pub fn chunk_coord_from_world_local_meters(&self, position_meters: [f32; 3]) -> ChunkCoord3 {
        self.chunk_coord_from_world_local_position(WorldLocalPosition::new(position_meters))
    }

    pub fn region_coord_from_chunk_coord(&self, chunk: ChunkCoord3) -> RegionCoord3 {
        let dims = self.region_chunk_dims_i32();

        RegionCoord3 {
            x: div_floor(chunk.x, dims[0]),
            y: div_floor(chunk.y, dims[1]),
            z: div_floor(chunk.z, dims[2]),
        }
    }

    pub fn chunk_id_from_position(
        &self,
        world_id: WorldId,
        position: WorldLocalPosition,
    ) -> ChunkId {
        ChunkId::new(
            world_id,
            self.chunk_coord_from_world_local_position(position),
        )
    }

    pub fn chunk_id_from_meters(&self, world_id: WorldId, position_meters: [f32; 3]) -> ChunkId {
        self.chunk_id_from_position(world_id, WorldLocalPosition::new(position_meters))
    }

    pub fn region_id_from_chunk_id(&self, chunk_id: ChunkId) -> RegionId {
        RegionId::new(
            chunk_id.world_id,
            self.region_coord_from_chunk_coord(chunk_id.coord),
        )
    }

    pub fn region_id_from_position(
        &self,
        world_id: WorldId,
        position: WorldLocalPosition,
    ) -> RegionId {
        self.region_id_from_chunk_id(self.chunk_id_from_position(world_id, position))
    }

    fn region_chunk_dims_i32(&self) -> [i32; 3] {
        [
            self.region_chunk_dims[0].max(1) as i32,
            self.region_chunk_dims[1].max(1) as i32,
            self.region_chunk_dims[2].max(1) as i32,
        ]
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

#[cfg(test)]
mod tests {
    use super::GridPartitionConfig;
    use crate::{ChunkCoord3, WorldId, WorldLocalPosition};

    #[test]
    fn region_coord_handles_negative_chunks() {
        let partition = GridPartitionConfig::default();
        let region = partition.region_coord_from_chunk_coord(ChunkCoord3 {
            x: -1,
            y: -8,
            z: -9,
        });

        assert_eq!(region.x, -1);
        assert_eq!(region.y, -1);
        assert_eq!(region.z, -2);
    }

    #[test]
    fn chunk_id_boundaries_floor_into_correct_chunk() {
        let partition = GridPartitionConfig {
            chunk_edge_meters: 10.0,
            ..GridPartitionConfig::default()
        };

        let world = WorldId(3);
        let chunk_a =
            partition.chunk_id_from_position(world, WorldLocalPosition::new([9.999, 0.0, -0.001]));
        let chunk_b =
            partition.chunk_id_from_position(world, WorldLocalPosition::new([10.0, 0.0, 0.0]));

        assert_eq!(chunk_a.coord.x, 0);
        assert_eq!(chunk_b.coord.x, 1);
        assert_eq!(chunk_a.world_id, world);
        assert_eq!(chunk_b.world_id, world);
    }
}
