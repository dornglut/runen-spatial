use runen_spatial::{
    ChunkCoord3, ClipmapConfig, ClipmapCoord3, ClipmapLevel, FrameLocalPosition, GridLevel,
    GridPartitionConfig, HierarchicalChunkId, HierarchicalGridConfig, RingBufferConfig,
    SpatialMathError, WorldFrame, WorldId, WorldPosition, clipmap_coord_from_world_position,
    clipmap_window_for_center, ring_slot_for_coord,
};

#[test]
fn positions_and_frames_are_finite_namespaced_and_translation_only() {
    assert!(matches!(
        WorldPosition::try_new(WorldId(1), [f64::NAN, 0.0, 0.0]),
        Err(SpatialMathError::NonFiniteValue { .. })
    ));
    assert!(matches!(
        FrameLocalPosition::try_new([f32::INFINITY, 0.0, 0.0]),
        Err(SpatialMathError::NonFiniteValue { .. })
    ));
    let origin = WorldPosition::try_new(WorldId(1), [10.0, -2.0, 3.0]).unwrap();
    let frame = WorldFrame::try_new(origin).unwrap();
    let global = WorldPosition::try_new(WorldId(1), [12.5, 1.0, 3.0]).unwrap();
    let local = frame.to_local(global).unwrap();
    assert_eq!(local.meters(), [2.5, 3.0, 0.0]);
    assert_eq!(frame.to_global(local).unwrap(), global);
    assert!(matches!(
        frame.to_local(WorldPosition::try_new(WorldId(2), [0.0; 3]).unwrap()),
        Err(SpatialMathError::WorldMismatch { .. })
    ));
    assert!(matches!(
        frame.to_local(
            WorldPosition::try_new(WorldId(1), [f64::from(f32::MAX) * 2.0, 0.0, 0.0]).unwrap()
        ),
        Err(SpatialMathError::LocalPositionOutOfRange { .. })
    ));
}

#[test]
fn partition_uses_checked_floor_conversion_and_negative_division() {
    let partition = GridPartitionConfig::try_new(10.0, [8, 8, 8]).unwrap();
    let position = WorldPosition::try_new(WorldId(3), [-0.001, 9.999, 10.0]).unwrap();
    assert_eq!(
        partition.chunk_coord_from_world_position(position).unwrap(),
        ChunkCoord3 { x: -1, y: 0, z: 1 }
    );
    assert_eq!(
        partition
            .region_coord_from_chunk_coord(ChunkCoord3 {
                x: -1,
                y: -8,
                z: -9
            })
            .unwrap()
            .z,
        -2
    );
    let lower =
        WorldPosition::try_new(WorldId(3), [-9_223_372_036_854_775_808.0, 0.0, 0.0]).unwrap();
    assert_eq!(
        GridPartitionConfig::try_new(1.0, [1, 1, 1])
            .unwrap()
            .chunk_coord_from_world_position(lower)
            .unwrap()
            .x,
        i64::MIN
    );
    let upper =
        WorldPosition::try_new(WorldId(3), [9_223_372_036_854_775_808.0, 0.0, 0.0]).unwrap();
    assert!(matches!(
        GridPartitionConfig::try_new(1.0, [1, 1, 1])
            .unwrap()
            .chunk_coord_from_world_position(upper),
        Err(SpatialMathError::CoordinateOutOfRange { .. })
    ));
    assert!(matches!(
        GridPartitionConfig::try_new(0.0, [1, 1, 1]),
        Err(SpatialMathError::NonPositiveValue { .. })
    ));
    assert!(matches!(
        GridPartitionConfig::try_new(1.0, [0, 1, 1]),
        Err(SpatialMathError::ZeroDimension { .. })
    ));
}

#[test]
fn hierarchy_is_finest_to_coarsest_with_checked_bounds() {
    let config = HierarchicalGridConfig::try_new(1.0, 3, 2).unwrap();
    assert_eq!(
        config.parent_level(GridLevel(0)).unwrap(),
        Some(GridLevel(1))
    );
    assert_eq!(config.child_level(GridLevel(0)).unwrap(), None);
    assert_eq!(
        config
            .parent_coord(GridLevel(0), ChunkCoord3 { x: -1, y: 3, z: 0 })
            .unwrap(),
        ChunkCoord3 { x: -1, y: 1, z: 0 }
    );
    assert_eq!(
        config
            .child_coord_bounds(GridLevel(1), ChunkCoord3 { x: -1, y: 0, z: 1 })
            .unwrap(),
        (
            ChunkCoord3 { x: -2, y: 0, z: 2 },
            ChunkCoord3 { x: -1, y: 1, z: 3 }
        )
    );
    assert_eq!(
        HierarchicalChunkId::new(WorldId(1), GridLevel(2), ChunkCoord3::default())
            .parent(&config)
            .unwrap(),
        None
    );
    assert!(matches!(
        config.first_child_coord(
            GridLevel(1),
            ChunkCoord3 {
                x: i64::MAX,
                y: 0,
                z: 0
            }
        ),
        Err(SpatialMathError::ArithmeticOverflow { .. })
    ));
    assert!(matches!(
        HierarchicalGridConfig::try_new(1.0, 0, 2),
        Err(SpatialMathError::LevelCountZero)
    ));
}

#[test]
fn clipmap_and_ring_are_checked_mapping_primitives() {
    let clipmap = ClipmapConfig::try_new(2.0, 2, 2, [3, 3, 3]).unwrap();
    let position = WorldPosition::try_new(WorldId(1), [-0.1, 0.0, 0.0]).unwrap();
    assert_eq!(
        clipmap_coord_from_world_position(&clipmap, ClipmapLevel(0), position)
            .unwrap()
            .x,
        -1
    );
    assert!(matches!(
        clipmap_window_for_center(
            &clipmap,
            ClipmapLevel(0),
            ClipmapCoord3 {
                x: i64::MAX,
                y: 0,
                z: 0
            }
        ),
        Err(SpatialMathError::ArithmeticOverflow { .. })
    ));
    assert!(matches!(
        ClipmapConfig::try_new(1.0, 1, 2, [2, 3, 3]),
        Err(SpatialMathError::EvenWindowDimension { .. })
    ));
    let ring = RingBufferConfig::try_new([17, 5, 17]).unwrap();
    let anchor = ClipmapCoord3 {
        x: i64::MIN,
        y: i64::MAX,
        z: 0,
    };
    let slot = ring_slot_for_coord(
        anchor,
        ClipmapCoord3 {
            x: i64::MAX,
            y: i64::MIN,
            z: -1,
        },
        &ring,
    );
    assert_eq!(
        slot,
        ring_slot_for_coord(
            anchor,
            ClipmapCoord3 {
                x: i64::MAX - 17,
                y: i64::MIN + 5,
                z: 16
            },
            &ring
        )
    );
    assert!(matches!(
        RingBufferConfig::try_new([0, 1, 1]),
        Err(SpatialMathError::ZeroDimension { .. })
    ));
}
