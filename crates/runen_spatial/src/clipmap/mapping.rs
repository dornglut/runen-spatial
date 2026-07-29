use crate::WorldLocalPosition;
use crate::clipmap::{ClipmapConfig, ClipmapCoord3, ClipmapLevel};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClipmapWindow {
    pub level: ClipmapLevel,
    pub center: ClipmapCoord3,
    pub min: ClipmapCoord3,
    pub max: ClipmapCoord3,
}

pub fn coord_from_world_local_position(
    config: &ClipmapConfig,
    level: ClipmapLevel,
    position: WorldLocalPosition,
) -> ClipmapCoord3 {
    let edge = config.cell_edge_meters_for_level(level.0).max(1.0);

    ClipmapCoord3 {
        x: (position.meters[0] / edge).floor() as i32,
        y: (position.meters[1] / edge).floor() as i32,
        z: (position.meters[2] / edge).floor() as i32,
    }
}

pub fn window_for_center(
    config: &ClipmapConfig,
    level: ClipmapLevel,
    center: ClipmapCoord3,
) -> ClipmapWindow {
    let dims = config.clamped_window_dims();
    let half_x = (dims[0] as i32) / 2;
    let half_y = (dims[1] as i32) / 2;
    let half_z = (dims[2] as i32) / 2;

    ClipmapWindow {
        level,
        center,
        min: ClipmapCoord3 {
            x: center.x - half_x,
            y: center.y - half_y,
            z: center.z - half_z,
        },
        max: ClipmapCoord3 {
            x: center.x + half_x,
            y: center.y + half_y,
            z: center.z + half_z,
        },
    }
}
