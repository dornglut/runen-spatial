use crate::{SpatialMathError, WorldId};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WorldPosition {
    world_id: WorldId,
    meters: [f64; 3],
}

impl WorldPosition {
    pub fn try_new(world_id: WorldId, meters: [f64; 3]) -> Result<Self, SpatialMathError> {
        for value in meters {
            if !value.is_finite() {
                return Err(SpatialMathError::NonFiniteValue {
                    field: "world position meters",
                });
            }
        }
        Ok(Self { world_id, meters })
    }

    pub fn world_id(self) -> WorldId {
        self.world_id
    }
    pub fn meters(self) -> [f64; 3] {
        self.meters
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FrameLocalPosition {
    meters: [f32; 3],
}

impl FrameLocalPosition {
    pub fn try_new(meters: [f32; 3]) -> Result<Self, SpatialMathError> {
        for value in meters {
            if !value.is_finite() {
                return Err(SpatialMathError::NonFiniteValue {
                    field: "frame-local position meters",
                });
            }
        }
        Ok(Self { meters })
    }

    pub fn meters(self) -> [f32; 3] {
        self.meters
    }
}
