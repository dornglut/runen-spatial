use crate::{WorldId, WorldLocalPosition, WorldPosition};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WorldFrame {
    pub world_id: WorldId,
    pub origin_meters: [f64; 3],
    pub basis: [[f32; 3]; 3],
}

impl Default for WorldFrame {
    fn default() -> Self {
        Self {
            world_id: WorldId(0),
            origin_meters: [0.0, 0.0, 0.0],
            basis: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CameraRelativeFrame {
    pub world_id: WorldId,
    pub camera_world_position: WorldPosition,
    pub camera_world_local_position: WorldLocalPosition,
    pub render_origin_offset_meters: [f32; 3],
}

impl Default for CameraRelativeFrame {
    fn default() -> Self {
        Self {
            world_id: WorldId(0),
            camera_world_position: WorldPosition::new([0.0, 0.0, 0.0]),
            camera_world_local_position: WorldLocalPosition::new([0.0, 0.0, 0.0]),
            render_origin_offset_meters: [0.0, 0.0, 0.0],
        }
    }
}

pub fn build_camera_relative_frame(
    world_frame: WorldFrame,
    camera_world_position: WorldPosition,
) -> CameraRelativeFrame {
    let local = [
        (camera_world_position.meters[0] - world_frame.origin_meters[0]) as f32,
        (camera_world_position.meters[1] - world_frame.origin_meters[1]) as f32,
        (camera_world_position.meters[2] - world_frame.origin_meters[2]) as f32,
    ];

    CameraRelativeFrame {
        world_id: world_frame.world_id,
        camera_world_position,
        camera_world_local_position: WorldLocalPosition::new(local),
        render_origin_offset_meters: local,
    }
}
