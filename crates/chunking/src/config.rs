use crate::{ChunkLoadOrder, ChunkStreamingMode};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChunkStreamingConfig {
    pub load_radius_chunks: i32,
    pub unload_radius_chunks: i32,
    pub vertical_load_radius_chunks: i32,
    pub vertical_unload_radius_chunks: i32,
    pub mode: ChunkStreamingMode,
    pub load_order: ChunkLoadOrder,
}

impl Default for ChunkStreamingConfig {
    fn default() -> Self {
        Self {
            load_radius_chunks: 4,
            unload_radius_chunks: 6,
            vertical_load_radius_chunks: 1,
            vertical_unload_radius_chunks: 2,
            mode: ChunkStreamingMode::PlanarXZ,
            load_order: ChunkLoadOrder::NearestFirst,
        }
    }
}

impl ChunkStreamingConfig {
    pub fn clamped(self) -> Self {
        let load_radius_chunks = self.load_radius_chunks.max(0);
        let vertical_load_radius_chunks = self.vertical_load_radius_chunks.max(0);

        Self {
            load_radius_chunks,
            unload_radius_chunks: self.unload_radius_chunks.max(load_radius_chunks),
            vertical_load_radius_chunks,
            vertical_unload_radius_chunks: self
                .vertical_unload_radius_chunks
                .max(vertical_load_radius_chunks),
            mode: self.mode,
            load_order: self.load_order,
        }
    }
}
