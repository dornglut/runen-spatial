#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WorldLocalPosition {
    pub meters: [f32; 3],
}

impl WorldLocalPosition {
    pub fn new(meters: [f32; 3]) -> Self {
        Self { meters }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WorldPosition {
    pub meters: [f64; 3],
}

impl WorldPosition {
    pub fn new(meters: [f64; 3]) -> Self {
        Self { meters }
    }
}
