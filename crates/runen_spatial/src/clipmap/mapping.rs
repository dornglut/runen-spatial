use crate::clipmap::{ClipmapConfig, ClipmapCoord3, ClipmapLevel};
use crate::grid::partition::coordinate_from_meters;
use crate::{SpatialMathError, WorldPosition};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClipmapWindow {
    pub level: ClipmapLevel,
    pub center: ClipmapCoord3,
    pub min: ClipmapCoord3,
    pub max: ClipmapCoord3,
}

pub fn coord_from_world_position(
    config: &ClipmapConfig,
    level: ClipmapLevel,
    position: WorldPosition,
) -> Result<ClipmapCoord3, SpatialMathError> {
    let edge = config.cell_edge_meters_for_level(level)?;
    let meters = position.meters();
    Ok(ClipmapCoord3 {
        x: coordinate_from_meters(meters[0], edge, "clipmap x")?,
        y: coordinate_from_meters(meters[1], edge, "clipmap y")?,
        z: coordinate_from_meters(meters[2], edge, "clipmap z")?,
    })
}

pub fn window_for_center(
    config: &ClipmapConfig,
    level: ClipmapLevel,
    center: ClipmapCoord3,
) -> Result<ClipmapWindow, SpatialMathError> {
    config.cell_edge_meters_for_level(level)?;
    let dims = config.window_dims();
    let half = [
        i64::from(dims[0] / 2),
        i64::from(dims[1] / 2),
        i64::from(dims[2] / 2),
    ];
    let min = ClipmapCoord3 {
        x: center
            .x
            .checked_sub(half[0])
            .ok_or(SpatialMathError::ArithmeticOverflow {
                operation: "clipmap window minimum",
            })?,
        y: center
            .y
            .checked_sub(half[1])
            .ok_or(SpatialMathError::ArithmeticOverflow {
                operation: "clipmap window minimum",
            })?,
        z: center
            .z
            .checked_sub(half[2])
            .ok_or(SpatialMathError::ArithmeticOverflow {
                operation: "clipmap window minimum",
            })?,
    };
    let max = ClipmapCoord3 {
        x: center
            .x
            .checked_add(half[0])
            .ok_or(SpatialMathError::ArithmeticOverflow {
                operation: "clipmap window maximum",
            })?,
        y: center
            .y
            .checked_add(half[1])
            .ok_or(SpatialMathError::ArithmeticOverflow {
                operation: "clipmap window maximum",
            })?,
        z: center
            .z
            .checked_add(half[2])
            .ok_or(SpatialMathError::ArithmeticOverflow {
                operation: "clipmap window maximum",
            })?,
    };
    Ok(ClipmapWindow {
        level,
        center,
        min,
        max,
    })
}
