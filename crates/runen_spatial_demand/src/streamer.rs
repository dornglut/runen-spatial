use crate::{
    ChunkLoadOrder, ChunkSet, ChunkSetDiff, ChunkStreamingConfig, ChunkStreamingMode,
    StreamingFocus,
};
use runen_spatial::{ChunkCoord3, GridPartitionConfig, SpatialMathError};

pub struct ChunkStreamer {
    partition: GridPartitionConfig,
    config: ChunkStreamingConfig,
    active: ChunkSet,
}

impl ChunkStreamer {
    pub fn new(partition: GridPartitionConfig, config: ChunkStreamingConfig) -> Self {
        Self {
            partition,
            config: config.clamped(),
            active: ChunkSet::default(),
        }
    }
    pub fn partition(&self) -> &GridPartitionConfig {
        &self.partition
    }
    pub fn config(&self) -> ChunkStreamingConfig {
        self.config
    }
    pub fn active_chunks(&self) -> &ChunkSet {
        &self.active
    }
    pub fn active_chunk_count(&self) -> usize {
        self.active.len()
    }
    pub fn set_config(&mut self, config: ChunkStreamingConfig) {
        self.config = config.clamped();
    }
    pub fn clear(&mut self) {
        self.active.clear();
    }

    pub fn center_chunk_for_focus(
        &self,
        focus: StreamingFocus,
    ) -> Result<ChunkCoord3, SpatialMathError> {
        self.partition
            .chunk_coord_from_world_position(focus.position())
    }

    pub fn update_focus(
        &mut self,
        focus: StreamingFocus,
    ) -> Result<ChunkSetDiff, SpatialMathError> {
        let center = self.center_chunk_for_focus(focus)?;
        let desired = self.build_chunk_set(
            center,
            self.config.load_radius_chunks,
            self.config.vertical_load_radius_chunks,
        )?;
        let retained = self.build_chunk_set(
            center,
            self.config.unload_radius_chunks,
            self.config.vertical_unload_radius_chunks,
        )?;
        let mut next = desired.clone();
        for chunk in self.active.iter() {
            if retained.contains(chunk) {
                next.insert(*chunk);
            }
        }
        let mut diff = diff_chunk_sets(&self.active, &next);
        sort_chunks(center, &mut diff.entered, self.config.load_order);
        sort_chunks(center, &mut diff.exited, self.config.load_order);
        self.active = next;
        Ok(diff)
    }

    pub fn desired_chunks_for_focus(
        &self,
        focus: StreamingFocus,
    ) -> Result<ChunkSet, SpatialMathError> {
        self.build_chunk_set(
            self.center_chunk_for_focus(focus)?,
            self.config.load_radius_chunks,
            self.config.vertical_load_radius_chunks,
        )
    }

    fn build_chunk_set(
        &self,
        center: ChunkCoord3,
        horizontal_radius: i32,
        vertical_radius: i32,
    ) -> Result<ChunkSet, SpatialMathError> {
        let horizontal = i64::from(horizontal_radius);
        let vertical = i64::from(vertical_radius);
        let x = checked_range(center.x, horizontal)?;
        let y = checked_range(center.y, vertical)?;
        let z = checked_range(center.z, horizontal)?;
        let mut set = ChunkSet::default();
        match self.config.mode {
            ChunkStreamingMode::PlanarXZ | ChunkStreamingMode::Volume3D => {
                for x in x.0..=x.1 {
                    for y in y.0..=y.1 {
                        for z in z.0..=z.1 {
                            set.insert(ChunkCoord3 { x, y, z });
                        }
                    }
                }
            }
        }
        Ok(set)
    }
}

fn checked_range(center: i64, radius: i64) -> Result<(i64, i64), SpatialMathError> {
    Ok((
        center
            .checked_sub(radius)
            .ok_or(SpatialMathError::ArithmeticOverflow {
                operation: "streaming chunk range",
            })?,
        center
            .checked_add(radius)
            .ok_or(SpatialMathError::ArithmeticOverflow {
                operation: "streaming chunk range",
            })?,
    ))
}

fn diff_chunk_sets(previous: &ChunkSet, next: &ChunkSet) -> ChunkSetDiff {
    let entered = next
        .iter()
        .filter(|chunk| !previous.contains(chunk))
        .copied()
        .collect();
    let exited = previous
        .iter()
        .filter(|chunk| !next.contains(chunk))
        .copied()
        .collect();
    ChunkSetDiff { entered, exited }
}

fn sort_chunks(center: ChunkCoord3, chunks: &mut [ChunkCoord3], order: ChunkLoadOrder) {
    chunks.sort_by_key(|chunk| {
        let dx = i128::from(chunk.x) - i128::from(center.x);
        let dy = i128::from(chunk.y) - i128::from(center.y);
        let dz = i128::from(chunk.z) - i128::from(center.z);
        dx * dx + dy * dy + dz * dz
    });
    if matches!(order, ChunkLoadOrder::FarthestFirst) {
        chunks.reverse();
    }
}
