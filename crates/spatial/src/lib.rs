pub mod bounds;
pub mod frames;
pub mod ids;
pub mod positions;

pub mod clipmap;
pub mod grid;
pub mod hash;
pub mod ring;

pub use bounds::{SpatialAabb3, SpatialPoint3};
pub use frames::{CameraRelativeFrame, WorldFrame, build_camera_relative_frame};
pub use ids::WorldId;
pub use positions::{WorldLocalPosition, WorldPosition};

pub use grid::{
    ChunkCoord3, ChunkId, GridLevel, GridPartitionConfig, HierarchicalChunkId,
    HierarchicalGridConfig, RegionCoord3, RegionId,
};

pub use clipmap::{
    ClipmapCellId, ClipmapConfig, ClipmapCoord3, ClipmapLevel, ClipmapWindow,
    coord_from_world_local_position as clipmap_coord_from_world_local_position,
    window_for_center as clipmap_window_for_center,
};

pub use ring::{RingBufferConfig, RingSlot3, slot_for_coord as ring_slot_for_coord};
