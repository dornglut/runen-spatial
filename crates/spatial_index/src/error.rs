#[derive(Debug, Clone, PartialEq)]
pub enum SpatialIndexError {
    InvalidBounds,
    InvalidCellSize { cell_size: f32 },
}
