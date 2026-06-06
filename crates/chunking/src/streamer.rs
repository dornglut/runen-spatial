use crate::{
    ChunkLoadOrder, ChunkSet, ChunkSetDiff, ChunkStreamingConfig, ChunkStreamingMode,
    StreamingFocus,
};
use spatial::{ChunkCoord3, GridPartitionConfig};

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

    pub fn center_chunk_for_focus(&self, focus: StreamingFocus) -> ChunkCoord3 {
        self.partition
            .chunk_coord_from_world_local_meters(focus.position_meters)
    }

    pub fn update_focus(&mut self, focus: StreamingFocus) -> ChunkSetDiff {
        let center = self.center_chunk_for_focus(focus);

        let desired = self.build_chunk_set(
            center,
            self.config.load_radius_chunks,
            self.config.vertical_load_radius_chunks,
        );

        let retained = self.build_chunk_set(
            center,
            self.config.unload_radius_chunks,
            self.config.vertical_unload_radius_chunks,
        );

        let mut next = desired.clone();
        for chunk in self.active.iter() {
            if retained.contains(chunk) {
                next.insert(*chunk);
            }
        }

        let mut diff = diff_chunk_sets(&self.active, &next);
        self.active = next;

        sort_chunks(center, &mut diff.entered, self.config.load_order);
        sort_chunks(center, &mut diff.exited, self.config.load_order);

        diff
    }

    pub fn desired_chunks_for_focus(&self, focus: StreamingFocus) -> ChunkSet {
        let center = self.center_chunk_for_focus(focus);
        self.build_chunk_set(
            center,
            self.config.load_radius_chunks,
            self.config.vertical_load_radius_chunks,
        )
    }

    fn build_chunk_set(
        &self,
        center: ChunkCoord3,
        horizontal_radius: i32,
        vertical_radius: i32,
    ) -> ChunkSet {
        let mut set = ChunkSet::default();

        match self.config.mode {
            ChunkStreamingMode::PlanarXZ => {
                for x in (center.x - horizontal_radius)..=(center.x + horizontal_radius) {
                    for z in (center.z - horizontal_radius)..=(center.z + horizontal_radius) {
                        for y in (center.y - vertical_radius)..=(center.y + vertical_radius) {
                            set.insert(ChunkCoord3 { x, y, z });
                        }
                    }
                }
            }
            ChunkStreamingMode::Volume3D => {
                for x in (center.x - horizontal_radius)..=(center.x + horizontal_radius) {
                    for y in (center.y - vertical_radius)..=(center.y + vertical_radius) {
                        for z in (center.z - horizontal_radius)..=(center.z + horizontal_radius) {
                            set.insert(ChunkCoord3 { x, y, z });
                        }
                    }
                }
            }
        }

        set
    }
}

fn diff_chunk_sets(previous: &ChunkSet, next: &ChunkSet) -> ChunkSetDiff {
    let mut entered = Vec::new();
    let mut exited = Vec::new();

    for chunk in next.iter() {
        if !previous.contains(chunk) {
            entered.push(*chunk);
        }
    }

    for chunk in previous.iter() {
        if !next.contains(chunk) {
            exited.push(*chunk);
        }
    }

    ChunkSetDiff { entered, exited }
}

fn sort_chunks(center: ChunkCoord3, chunks: &mut [ChunkCoord3], order: ChunkLoadOrder) {
    chunks.sort_by_key(|chunk| {
        let dx = chunk.x - center.x;
        let dy = chunk.y - center.y;
        let dz = chunk.z - center.z;
        dx * dx + dy * dy + dz * dz
    });

    if matches!(order, ChunkLoadOrder::FarthestFirst) {
        chunks.reverse();
    }
}
