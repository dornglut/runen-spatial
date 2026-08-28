use crate::{FrameLocalPosition, SpatialMathError, WorldPosition};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WorldFrame {
    origin: WorldPosition,
}

impl WorldFrame {
    pub fn new(origin: WorldPosition) -> Self {
        Self { origin }
    }

    pub fn origin(self) -> WorldPosition {
        self.origin
    }

    pub fn to_local(self, position: WorldPosition) -> Result<FrameLocalPosition, SpatialMathError> {
        if position.world_id() != self.origin.world_id() {
            return Err(SpatialMathError::WorldMismatch {
                expected: self.origin.world_id(),
                actual: position.world_id(),
            });
        }
        let global = position.meters();
        let origin = self.origin.meters();
        let mut local = [0.0; 3];
        for axis in 0..3 {
            let delta = global[axis] - origin[axis];
            if !delta.is_finite() || delta.abs() > f64::from(f32::MAX) {
                return Err(SpatialMathError::LocalPositionOutOfRange { axis: axis as u8 });
            }
            local[axis] = delta as f32;
        }
        FrameLocalPosition::try_new(local)
    }

    pub fn to_global(self, local: FrameLocalPosition) -> Result<WorldPosition, SpatialMathError> {
        let origin = self.origin.meters();
        let meters = local.meters();
        let mut global = [0.0; 3];
        for axis in 0..3 {
            global[axis] = origin[axis] + f64::from(meters[axis]);
            if !global[axis].is_finite() {
                return Err(SpatialMathError::ArithmeticOverflow {
                    operation: "frame local to global",
                });
            }
        }
        WorldPosition::try_new(self.origin.world_id(), global)
    }
}
