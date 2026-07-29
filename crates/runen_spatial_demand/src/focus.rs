use runen_spatial::WorldPosition;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StreamingFocus {
    position: WorldPosition,
}

impl StreamingFocus {
    pub fn new(position: WorldPosition) -> Self {
        Self { position }
    }
    pub fn position(self) -> WorldPosition {
        self.position
    }
}
