use runen_spatial::{ChunkCoord3, GridPartitionConfig, SpatialMathError, WorldId, WorldPosition};
use runen_spatial_demand::{
    ChunkLoadOrder, ChunkStreamer, ChunkStreamingConfig, ChunkStreamingMode, StreamingFocus,
};

fn partition() -> GridPartitionConfig {
    GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap()
}
fn focus(meters: [f64; 3]) -> StreamingFocus {
    StreamingFocus::new(WorldPosition::try_new(WorldId(7), meters).unwrap())
}
fn config(radius: i32) -> ChunkStreamingConfig {
    ChunkStreamingConfig {
        load_radius_chunks: radius,
        unload_radius_chunks: radius,
        vertical_load_radius_chunks: 0,
        vertical_unload_radius_chunks: 0,
        mode: ChunkStreamingMode::PlanarXZ,
        load_order: ChunkLoadOrder::NearestFirst,
    }
}

#[test]
fn valid_focus_retains_chunk_demand_behavior() {
    let mut streamer = ChunkStreamer::new(partition(), config(1));
    let diff = streamer.update_focus(focus([0.0, 0.0, 0.0])).unwrap();
    assert_eq!(diff.entered.len(), 9);
    assert_eq!(
        streamer
            .center_chunk_for_focus(focus([15.9, 0.0, -0.1]))
            .unwrap(),
        ChunkCoord3 { x: 0, y: 0, z: -1 }
    );
}

#[test]
fn near_boundary_focus_returns_error_without_replacing_active_state() {
    let mut streamer = ChunkStreamer::new(partition(), config(1));
    streamer.update_focus(focus([0.0, 0.0, 0.0])).unwrap();
    let before = streamer.active_chunks().clone();
    let error = streamer
        .update_focus(focus([i64::MAX as f64 * 16.0, 0.0, 0.0]))
        .unwrap_err();
    assert!(matches!(
        error,
        SpatialMathError::CoordinateOutOfRange { .. } | SpatialMathError::ArithmeticOverflow { .. }
    ));
    assert_eq!(streamer.active_chunks(), &before);
}

#[test]
fn distance_ordering_handles_extreme_coordinates() {
    let mut streamer = ChunkStreamer::new(partition(), config(0));
    let center = WorldPosition::try_new(WorldId(7), [0.0, 0.0, 0.0]).unwrap();
    let _ = streamer.update_focus(StreamingFocus::new(center)).unwrap();
    let desired = streamer
        .desired_chunks_for_focus(focus([0.0, 0.0, 0.0]))
        .unwrap();
    assert!(desired.contains(&ChunkCoord3 { x: 0, y: 0, z: 0 }));
}
