use runen_spatial::{ChunkCoord3, GridPartitionConfig, SpatialMathError, WorldId, WorldPosition};
use runen_spatial_demand::{
    ChunkLoadOrder, ChunkStreamingConfig, ChunkStreamingMode, StreamingFocus,
};
use runen_spatial_streaming::{
    StreamingBudgets, StreamingTick, WorldStreamingConfig, WorldStreamingController,
    WorldStreamingError,
};

fn controller() -> WorldStreamingController {
    let mut config = WorldStreamingConfig::new(
        WorldId(7),
        GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap(),
        ChunkStreamingConfig {
            load_radius_chunks: 0,
            unload_radius_chunks: 0,
            vertical_load_radius_chunks: 0,
            vertical_unload_radius_chunks: 0,
            mode: ChunkStreamingMode::PlanarXZ,
            load_order: ChunkLoadOrder::NearestFirst,
        },
    );
    config.budgets = StreamingBudgets {
        max_load_requests_per_tick: 1,
        max_unload_requests_per_tick: 1,
    };
    WorldStreamingController::new(config)
}
fn tick(world: WorldId, meters: [f64; 3]) -> StreamingTick {
    StreamingTick::from_focus(StreamingFocus::new(
        WorldPosition::try_new(world, meters).unwrap(),
    ))
}

#[test]
fn valid_tick_preserves_request_emission() {
    let mut controller = controller();
    let output = controller.tick(tick(WorldId(7), [0.0, 0.0, 0.0])).unwrap();
    assert_eq!(output.requests.len(), 1);
    assert_eq!(
        output.requests[0].chunk_id.coord,
        ChunkCoord3 { x: 0, y: 0, z: 0 }
    );
}

#[test]
fn mismatched_world_fails_without_mutating_records() {
    let mut controller = controller();
    let error = controller
        .tick(tick(WorldId(8), [0.0, 0.0, 0.0]))
        .unwrap_err();
    assert_eq!(
        error,
        WorldStreamingError::SpatialMath(SpatialMathError::WorldMismatch {
            expected: WorldId(7),
            actual: WorldId(8)
        })
    );
    assert_eq!(controller.records().count(), 0);
    assert_eq!(controller.pending_requests().count(), 0);
}

#[test]
fn spatial_failure_does_not_partially_mutate_lifecycle() {
    let mut controller = controller();
    let error = controller
        .tick(tick(WorldId(7), [i64::MAX as f64 * 16.0, 0.0, 0.0]))
        .unwrap_err();
    assert!(matches!(
        error,
        WorldStreamingError::SpatialMath(SpatialMathError::CoordinateOutOfRange { .. })
    ));
    assert_eq!(controller.records().count(), 0);
    assert_eq!(controller.pending_requests().count(), 0);
}
