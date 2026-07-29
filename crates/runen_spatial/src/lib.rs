pub mod bounds;
pub mod error;
pub mod frames;
pub mod ids;
pub mod positions;

pub mod clipmap;
pub mod grid;
pub mod hash;
pub mod ring;

pub use bounds::{SpatialAabb3, SpatialPoint3};
pub use error::SpatialMathError;
pub use frames::WorldFrame;
pub use hash::{
    SpatialHashSeed, SpatialHashValue, finalize_u64 as finalize_spatial_hash,
    hash_cell2 as spatial_hash_cell2, hash_cell3 as spatial_hash_cell3,
    hash_i64s as spatial_hash_i64s, mix_i64 as mix_spatial_hash_i64,
    mix_u64 as mix_spatial_hash_u64,
};
pub use ids::WorldId;
pub use positions::{FrameLocalPosition, WorldPosition};

pub use grid::{
    ChunkCoord3, ChunkId, GridLevel, GridPartitionConfig, HierarchicalChunkId,
    HierarchicalGridConfig, RegionCoord3, RegionId,
};

pub use clipmap::{
    ClipmapCellId, ClipmapConfig, ClipmapCoord3, ClipmapLevel, ClipmapWindow,
    coord_from_world_position as clipmap_coord_from_world_position,
    window_for_center as clipmap_window_for_center,
};

pub use ring::{RingBufferConfig, RingSlot3, slot_for_coord as ring_slot_for_coord};
