#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StreamingFocus {
    pub position_meters: [f32; 3],
}

impl StreamingFocus {
    pub fn new(position_meters: [f32; 3]) -> Self {
        Self { position_meters }
    }
}
