use runen_spatial::{ChunkCoord3, GridPartitionConfig, SpatialMathError, WorldId, WorldPosition};

use runen_spatial_demand::{
    ChunkLoadOrder, ChunkStreamer, ChunkStreamingConfig, ChunkStreamingMode, StreamingFocus,
};

fn default_partition() -> GridPartitionConfig {
    GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap()
}

fn focus(meters: [f64; 3]) -> StreamingFocus {
    StreamingFocus::new(WorldPosition::try_new(WorldId(7), meters).unwrap())
}

#[test]
fn first_update_populates_active_set() {
    let partition = default_partition();
    let config = ChunkStreamingConfig {
        load_radius_chunks: 1,
        unload_radius_chunks: 2,
        vertical_load_radius_chunks: 0,
        vertical_unload_radius_chunks: 0,
        mode: ChunkStreamingMode::PlanarXZ,
        load_order: ChunkLoadOrder::NearestFirst,
    };

    let mut streamer = ChunkStreamer::new(partition, config);

    let diff = streamer.update_focus(focus([0.0, 0.0, 0.0])).unwrap();

    assert_eq!(diff.exited.len(), 0);
    assert_eq!(diff.entered.len(), 9);
    assert_eq!(streamer.active_chunk_count(), 9);
}

#[test]
fn center_chunk_matches_partition_flooring() {
    let partition = default_partition();
    let streamer = ChunkStreamer::new(partition, ChunkStreamingConfig::default());

    let center = streamer
        .center_chunk_for_focus(focus([15.9, 0.0, -0.1]))
        .unwrap();

    assert_eq!(center, ChunkCoord3 { x: 0, y: 0, z: -1 });
}

#[test]
fn moving_focus_enters_and_exits_chunks() {
    let partition = default_partition();
    let config = ChunkStreamingConfig {
        load_radius_chunks: 0,
        unload_radius_chunks: 0,
        vertical_load_radius_chunks: 0,
        vertical_unload_radius_chunks: 0,
        mode: ChunkStreamingMode::PlanarXZ,
        load_order: ChunkLoadOrder::NearestFirst,
    };

    let mut streamer = ChunkStreamer::new(partition, config);

    let first = streamer.update_focus(focus([0.0, 0.0, 0.0])).unwrap();
    assert_eq!(first.entered, vec![ChunkCoord3 { x: 0, y: 0, z: 0 }]);
    assert!(first.exited.is_empty());

    let second = streamer.update_focus(focus([16.0, 0.0, 0.0])).unwrap();
    assert_eq!(second.entered, vec![ChunkCoord3 { x: 1, y: 0, z: 0 }]);
    assert_eq!(second.exited, vec![ChunkCoord3 { x: 0, y: 0, z: 0 }]);
}

#[test]
fn unload_hysteresis_retains_chunks_inside_unload_radius() {
    let partition = default_partition();
    let config = ChunkStreamingConfig {
        load_radius_chunks: 0,
        unload_radius_chunks: 1,
        vertical_load_radius_chunks: 0,
        vertical_unload_radius_chunks: 0,
        mode: ChunkStreamingMode::PlanarXZ,
        load_order: ChunkLoadOrder::NearestFirst,
    };

    let mut streamer = ChunkStreamer::new(partition, config);

    streamer.update_focus(focus([0.0, 0.0, 0.0])).unwrap();
    let diff = streamer.update_focus(focus([16.0, 0.0, 0.0])).unwrap();

    assert_eq!(diff.entered, vec![ChunkCoord3 { x: 1, y: 0, z: 0 }]);
    assert!(diff.exited.is_empty());
    assert!(
        streamer
            .active_chunks()
            .contains(&ChunkCoord3 { x: 0, y: 0, z: 0 })
    );
    assert!(
        streamer
            .active_chunks()
            .contains(&ChunkCoord3 { x: 1, y: 0, z: 0 })
    );
}

#[test]
fn planar_xz_uses_horizontal_footprint_with_vertical_band() {
    let partition = default_partition();
    let config = ChunkStreamingConfig {
        load_radius_chunks: 1,
        unload_radius_chunks: 1,
        vertical_load_radius_chunks: 1,
        vertical_unload_radius_chunks: 1,
        mode: ChunkStreamingMode::PlanarXZ,
        load_order: ChunkLoadOrder::NearestFirst,
    };

    let streamer = ChunkStreamer::new(partition, config);
    let desired = streamer
        .desired_chunks_for_focus(focus([0.0, 0.0, 0.0]))
        .unwrap();

    // (2*1+1) * (2*1+1) * (2*1+1) = 27
    assert_eq!(desired.len(), 27);
}

#[test]
fn volume3d_uses_full_xyz_volume() {
    let partition = default_partition();
    let config = ChunkStreamingConfig {
        load_radius_chunks: 1,
        unload_radius_chunks: 1,
        vertical_load_radius_chunks: 2,
        vertical_unload_radius_chunks: 2,
        mode: ChunkStreamingMode::Volume3D,
        load_order: ChunkLoadOrder::NearestFirst,
    };

    let streamer = ChunkStreamer::new(partition, config);
    let desired = streamer
        .desired_chunks_for_focus(focus([0.0, 0.0, 0.0]))
        .unwrap();

    // x: 3, y: 5, z: 3 => 45
    assert_eq!(desired.len(), 45);
}

#[test]
fn nearest_first_orders_by_distance() {
    let partition = default_partition();
    let config = ChunkStreamingConfig {
        load_radius_chunks: 1,
        unload_radius_chunks: 1,
        vertical_load_radius_chunks: 0,
        vertical_unload_radius_chunks: 0,
        mode: ChunkStreamingMode::PlanarXZ,
        load_order: ChunkLoadOrder::NearestFirst,
    };

    let mut streamer = ChunkStreamer::new(partition, config);
    let diff = streamer.update_focus(focus([0.0, 0.0, 0.0])).unwrap();

    assert_eq!(
        diff.entered.first().copied(),
        Some(ChunkCoord3 { x: 0, y: 0, z: 0 })
    );
}

#[test]
fn farthest_first_reverses_priority() {
    let partition = default_partition();
    let config = ChunkStreamingConfig {
        load_radius_chunks: 1,
        unload_radius_chunks: 1,
        vertical_load_radius_chunks: 0,
        vertical_unload_radius_chunks: 0,
        mode: ChunkStreamingMode::PlanarXZ,
        load_order: ChunkLoadOrder::FarthestFirst,
    };

    let mut streamer = ChunkStreamer::new(partition, config);
    let diff = streamer.update_focus(focus([0.0, 0.0, 0.0])).unwrap();

    assert_ne!(
        diff.entered.first().copied(),
        Some(ChunkCoord3 { x: 0, y: 0, z: 0 })
    );
    assert_eq!(
        diff.entered.last().copied(),
        Some(ChunkCoord3 { x: 0, y: 0, z: 0 })
    );
}

#[test]
fn config_is_clamped_on_creation() {
    let partition = default_partition();
    let config = ChunkStreamingConfig {
        load_radius_chunks: -4,
        unload_radius_chunks: -1,
        vertical_load_radius_chunks: -2,
        vertical_unload_radius_chunks: -1,
        mode: ChunkStreamingMode::PlanarXZ,
        load_order: ChunkLoadOrder::NearestFirst,
    };

    let streamer = ChunkStreamer::new(partition, config);
    let effective = streamer.config();

    assert_eq!(effective.load_radius_chunks, 0);
    assert_eq!(effective.unload_radius_chunks, 0);
    assert_eq!(effective.vertical_load_radius_chunks, 0);
    assert_eq!(effective.vertical_unload_radius_chunks, 0);
}

#[test]
fn full_range_distance_failure_is_atomic() {
    let mut streamer = ChunkStreamer::new(
        default_partition(),
        ChunkStreamingConfig {
            load_radius_chunks: 0,
            unload_radius_chunks: 0,
            vertical_load_radius_chunks: 0,
            vertical_unload_radius_chunks: 0,
            mode: ChunkStreamingMode::PlanarXZ,
            load_order: ChunkLoadOrder::NearestFirst,
        },
    );
    let old = -(2_f64.powi(62) * 16.0);
    let new = 2_f64.powi(62) * 16.0;
    streamer.update_focus(focus([old, old, old])).unwrap();
    let active_before = streamer.active_chunks().clone();

    let error = streamer.update_focus(focus([new, new, new])).unwrap_err();

    assert!(matches!(
        error,
        SpatialMathError::ArithmeticOverflow {
            operation: "chunk distance square" | "chunk distance sum"
        }
    ));
    assert_eq!(streamer.active_chunks(), &active_before);
}
