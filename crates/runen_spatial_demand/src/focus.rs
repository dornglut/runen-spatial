use crate::{DemandAxis, SpatialDemandError};
use runen_spatial::WorldPosition;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DemandFocus {
    position: WorldPosition,
    horizontal_desired_radius: u32,
    horizontal_retain_radius: u32,
    vertical_desired_radius: u32,
    vertical_retain_radius: u32,
}

impl DemandFocus {
    pub fn try_new(
        position: WorldPosition,
        horizontal_desired_radius: u32,
        horizontal_retain_radius: u32,
        vertical_desired_radius: u32,
        vertical_retain_radius: u32,
    ) -> Result<Self, SpatialDemandError> {
        if horizontal_retain_radius < horizontal_desired_radius {
            return Err(SpatialDemandError::RetainRadiusBelowDesired {
                axis: DemandAxis::Horizontal,
                desired: horizontal_desired_radius,
                retain: horizontal_retain_radius,
            });
        }
        if vertical_retain_radius < vertical_desired_radius {
            return Err(SpatialDemandError::RetainRadiusBelowDesired {
                axis: DemandAxis::Vertical,
                desired: vertical_desired_radius,
                retain: vertical_retain_radius,
            });
        }
        Ok(Self {
            position,
            horizontal_desired_radius,
            horizontal_retain_radius,
            vertical_desired_radius,
            vertical_retain_radius,
        })
    }

    pub const fn position(&self) -> WorldPosition {
        self.position
    }

    pub const fn horizontal_desired_radius(&self) -> u32 {
        self.horizontal_desired_radius
    }

    pub const fn horizontal_retain_radius(&self) -> u32 {
        self.horizontal_retain_radius
    }

    pub const fn vertical_desired_radius(&self) -> u32 {
        self.vertical_desired_radius
    }

    pub const fn vertical_retain_radius(&self) -> u32 {
        self.vertical_retain_radius
    }
}
