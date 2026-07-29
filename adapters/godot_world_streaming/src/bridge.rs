use godot::builtin::Vector3;
use runen_spatial::{ChunkCoord3, ChunkId, WorldId};
use runen_spatial_streaming::{ProviderEvent, ProviderEventKind, StreamRequestId};

pub fn vector3_to_meters(value: Vector3) -> [f64; 3] {
    [f64::from(value.x), f64::from(value.y), f64::from(value.z)]
}

pub fn chunk_coord_from_xyz(x: i64, y: i64, z: i64) -> ChunkCoord3 {
    ChunkCoord3 { x, y, z }
}

pub fn chunk_id_from_xyz(world_id: u16, x: i64, y: i64, z: i64) -> ChunkId {
    ChunkId::new(WorldId(world_id), chunk_coord_from_xyz(x, y, z))
}

pub fn provider_event_from_godot(
    world_id: u16,
    request_id: i64,
    x: i64,
    y: i64,
    z: i64,
    kind: ProviderEventKind,
) -> Option<ProviderEvent> {
    let request_id = u64::try_from(request_id).ok()?;
    Some(ProviderEvent {
        request_id: StreamRequestId(request_id),
        chunk_id: chunk_id_from_xyz(world_id, x, y, z),
        kind,
    })
}
