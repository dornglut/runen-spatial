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
    ChunkId::new(WorldId::new(world_id), chunk_coord_from_xyz(x, y, z))
}

pub fn provider_event_from_godot(
    world_id: u16,
    request_id: i64,
    x: i64,
    y: i64,
    z: i64,
    kind: ProviderEventKind,
) -> Option<ProviderEvent> {
    let request_id = u64::try_from(request_id)
        .ok()
        .and_then(StreamRequestId::try_new)?;
    Some(ProviderEvent {
        request_id,
        chunk_id: chunk_id_from_xyz(world_id, x, y, z),
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::provider_event_from_godot;
    use runen_spatial_streaming::ProviderEventKind;

    #[test]
    fn provider_event_rejects_nonpositive_request_ids() {
        for request_id in [-1, 0] {
            assert!(
                provider_event_from_godot(7, request_id, 1, 2, 3, ProviderEventKind::Completed,)
                    .is_none()
            );
        }
    }

    #[test]
    fn provider_event_preserves_positive_request_ids_exactly() {
        let event =
            provider_event_from_godot(7, i64::MAX, 1, 2, 3, ProviderEventKind::Completed).unwrap();
        assert_eq!(event.request_id.get(), i64::MAX as u64);
    }
}
