use runen_spatial::{
    ChunkCoord3, ChunkId, ClipmapConfig, ClipmapCoord3, ClipmapLevel, FrameLocalPosition,
    GridLevel, GridPartitionConfig, HierarchicalChunkId, HierarchicalGridConfig, RingBufferConfig,
    SpatialMathError, WorldFrame, WorldId, WorldPosition, clipmap_coord_from_world_position,
    clipmap_window_for_center, ring_slot_for_coord,
};
use serde::Deserialize;
use serde::de::{Deserializer, IntoDeserializer, Visitor, value};

#[derive(Clone)]
enum TestValue {
    F64(f64),
    U8(u8),
    U32(u32),
    U32Array([u32; 3]),
}

impl<'de> IntoDeserializer<'de, value::Error> for TestValue {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> Deserializer<'de> for TestValue {
    type Error = value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::F64(value) => visitor.visit_f64(value),
            Self::U8(value) => visitor.visit_u8(value),
            Self::U32(value) => visitor.visit_u32(value),
            Self::U32Array(values) => visitor.visit_seq(value::SeqDeserializer::new(
                values.into_iter().map(IntoDeserializer::into_deserializer),
            )),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier
        ignored_any
    }
}

fn deserialize_config<T>(fields: Vec<(&'static str, TestValue)>) -> Result<T, value::Error>
where
    T: for<'de> Deserialize<'de>,
{
    T::deserialize(value::MapDeserializer::new(fields.into_iter()))
}

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
    let frame = WorldFrame::new(origin);
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

    let precise_global = WorldPosition::try_new(WorldId(1), [10.1, -1.7, 3.333_333_3]).unwrap();
    let precise_local = frame.to_local(precise_global).unwrap();
    let round_trip = frame.to_global(precise_local).unwrap();
    for axis in 0..3 {
        let expected = precise_global.meters()[axis];
        let tolerance =
            f64::from(f32::EPSILON) * f64::from(precise_local.meters()[axis].abs().max(1.0));
        assert!((round_trip.meters()[axis] - expected).abs() <= tolerance);
    }
}

#[test]
fn chunk_origin_preserves_the_stable_coordinate_or_rejects_precision_loss() {
    let coordinates = [
        i64::MIN,
        -(1_i64 << 53) - 1,
        -(1_i64 << 53),
        -(1_i64 << 53) + 1,
        (1_i64 << 53) - 1,
        1_i64 << 53,
        (1_i64 << 53) + 1,
        i64::MAX,
    ];
    for edge in [0.5, 1.0, 16.0, 32.0] {
        let partition = GridPartitionConfig::try_new(edge, [1, 1, 1]).unwrap();
        for coordinate in coordinates {
            let chunk = ChunkCoord3 {
                x: coordinate,
                y: coordinate,
                z: coordinate,
            };
            match partition.chunk_origin_world_position(WorldId(3), chunk) {
                Ok(position) => assert_eq!(
                    partition.chunk_coord_from_world_position(position).unwrap(),
                    chunk
                ),
                Err(
                    SpatialMathError::PrecisionLoss { .. }
                    | SpatialMathError::CoordinateOutOfRange { .. }
                    | SpatialMathError::ArithmeticOverflow { .. },
                ) => {}
                Err(error) => panic!("unexpected chunk-origin error: {error:?}"),
            }
        }
    }
}

#[test]
fn partition_uses_checked_floor_conversion_and_negative_division() {
    let partition = GridPartitionConfig::try_new(10.0, [8, 8, 8]).unwrap();
    let position = WorldPosition::try_new(WorldId(3), [-0.001, 9.999, 10.0]).unwrap();
    assert_eq!(
        partition.chunk_coord_from_world_position(position).unwrap(),
        ChunkCoord3 { x: -1, y: 0, z: 1 }
    );
    let chunk = ChunkCoord3 {
        x: -1,
        y: -8,
        z: -9,
    };
    let region = partition.region_coord_from_chunk_coord(chunk);
    assert_eq!(region.z, -2);
    let region_id = partition.region_id_from_chunk_id(ChunkId::new(WorldId(3), chunk));
    assert_eq!(region_id.world_id, WorldId(3));
    assert_eq!(region_id.coord, region);
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
            .parent_coord(GridLevel(2), ChunkCoord3::default())
            .unwrap(),
        None
    );
    assert_eq!(
        config
            .first_child_coord(GridLevel(0), ChunkCoord3::default())
            .unwrap(),
        None
    );
    assert_eq!(
        config
            .child_coord_bounds(GridLevel(0), ChunkCoord3::default())
            .unwrap(),
        None
    );
    assert_eq!(
        config
            .parent_coord(GridLevel(0), ChunkCoord3 { x: -1, y: 3, z: 0 })
            .unwrap(),
        Some(ChunkCoord3 { x: -1, y: 1, z: 0 })
    );
    assert_eq!(
        config
            .child_coord_bounds(GridLevel(1), ChunkCoord3 { x: -1, y: 0, z: 1 })
            .unwrap(),
        Some((
            ChunkCoord3 { x: -2, y: 0, z: 2 },
            ChunkCoord3 { x: -1, y: 1, z: 3 }
        ))
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
    assert!(matches!(
        config.parent_coord(GridLevel(3), ChunkCoord3::default()),
        Err(SpatialMathError::LevelOutOfRange { .. })
    ));
    assert!(matches!(
        config.first_child_coord(GridLevel(3), ChunkCoord3::default()),
        Err(SpatialMathError::LevelOutOfRange { .. })
    ));
}

#[test]
fn hierarchy_child_bounds_cover_each_parent_without_overlap() {
    for scale in [2, 3, 4] {
        let config = HierarchicalGridConfig::try_new(1.0, 3, scale).unwrap();
        for x in -17..=17 {
            for y in -17..=17 {
                for z in -17..=17 {
                    let parent = ChunkCoord3 { x, y, z };
                    let (minimum, maximum) = config
                        .child_coord_bounds(GridLevel(1), parent)
                        .unwrap()
                        .unwrap();
                    let mut count = 0_u32;
                    for child_x in minimum.x..=maximum.x {
                        for child_y in minimum.y..=maximum.y {
                            for child_z in minimum.z..=maximum.z {
                                count += 1;
                                assert_eq!(
                                    config
                                        .parent_coord(
                                            GridLevel(0),
                                            ChunkCoord3 {
                                                x: child_x,
                                                y: child_y,
                                                z: child_z,
                                            },
                                        )
                                        .unwrap(),
                                    Some(parent)
                                );
                            }
                        }
                    }
                    assert_eq!(count, scale.pow(3));
                    let next_parent = ChunkCoord3 { x: x + 1, y, z };
                    let (next_minimum, _) = config
                        .child_coord_bounds(GridLevel(1), next_parent)
                        .unwrap()
                        .unwrap();
                    assert!(maximum.x < next_minimum.x);
                }
            }
        }
    }
}

#[test]
fn clipmap_and_ring_are_checked_mapping_primitives() {
    let clipmap = ClipmapConfig::try_new(2.0, 2, 2, [3, 3, 3]).unwrap();
    assert_eq!(
        clipmap
            .cell_edge_meters_for_level(ClipmapLevel(1))
            .unwrap(),
        4.0
    );
    assert_eq!(
        clipmap
            .cell_edge_meters_for_level(ClipmapLevel(2))
            .unwrap_err(),
        SpatialMathError::LevelOutOfRange {
            level: 2,
            level_count: 2,
        }
    );
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

#[test]
fn checked_config_deserialization_rejects_invalid_values() {
    assert!(
        deserialize_config::<HierarchicalGridConfig>(vec![
            ("base_chunk_edge_meters", TestValue::F64(1.0)),
            ("level_count", TestValue::U8(0)),
            ("level_scale_factor", TestValue::U32(2)),
        ])
        .is_err()
    );
    assert!(
        deserialize_config::<ClipmapConfig>(vec![
            ("base_cell_edge_meters", TestValue::F64(1.0)),
            ("level_count", TestValue::U8(1)),
            ("level_scale_factor", TestValue::U32(2)),
            ("window_dims", TestValue::U32Array([2, 3, 3])),
        ])
        .is_err()
    );
    assert!(
        deserialize_config::<RingBufferConfig>(vec![("dims", TestValue::U32Array([0, 1, 1]),)])
            .is_err()
    );
}
