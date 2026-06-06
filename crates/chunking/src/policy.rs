#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChunkStreamingMode {
    /// Chunking optimized for terrain/open-world style layouts:
    /// wide XZ footprint, narrower vertical band.
    PlanarXZ,

    /// Full volumetric chunk neighborhood.
    Volume3D,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChunkLoadOrder {
    NearestFirst,
    FarthestFirst,
}
