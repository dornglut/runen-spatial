use runen_spatial::{ChunkCoord3, GridPartitionConfig, WorldId, WorldPosition};
use runen_spatial_demand::{
    DemandClass, DemandDistanceOrder, DemandFocus, DemandLimits, DemandSourceChange,
    DemandSourceId, DemandSourcePriority, DemandSourceSnapshot, DemandTransaction,
    SpatialDemandError, SpatialDemandPlanner,
};

fn planner(limits: DemandLimits) -> SpatialDemandPlanner {
    SpatialDemandPlanner::new(
        WorldId(7),
        GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap(),
        limits,
    )
}
fn focus(
    world: WorldId,
    meters: [f64; 3],
    desired: u32,
    retain: u32,
    order: DemandDistanceOrder,
) -> DemandFocus {
    DemandFocus::try_new(
        WorldPosition::try_new(world, meters).unwrap(),
        desired,
        retain,
        0,
        0,
        order,
    )
    .unwrap()
}
fn snapshot(
    priority: u32,
    focus: Option<DemandFocus>,
    pins: impl IntoIterator<Item = ChunkCoord3>,
) -> DemandSourceSnapshot {
    DemandSourceSnapshot::try_new(DemandSourcePriority::new(priority), focus, pins).unwrap()
}
fn transaction(changes: impl IntoIterator<Item = DemandSourceChange>) -> DemandTransaction {
    DemandTransaction::try_new(changes).unwrap()
}

#[test]
fn constructors_reject_invalid_demand_contracts() {
    assert!(DemandLimits::try_new(0, 1, 1, 1).is_err());
    assert!(
        DemandFocus::try_new(
            WorldPosition::try_new(WorldId(7), [0.0; 3]).unwrap(),
            2,
            1,
            0,
            0,
            DemandDistanceOrder::NearestFirst
        )
        .is_err()
    );
    assert!(DemandSourceSnapshot::try_new(DemandSourcePriority::new(0), None, []).is_err());
    assert!(matches!(
        DemandTransaction::try_new([
            DemandSourceChange::Remove {
                source_id: DemandSourceId::new(1)
            },
            DemandSourceChange::Remove {
                source_id: DemandSourceId::new(1)
            }
        ]),
        Err(SpatialDemandError::DuplicateSourceChange { .. })
    ));
}

#[test]
fn complete_replacement_and_removal_are_deterministic() {
    let mut planner = planner(DemandLimits::default());
    let source = DemandSourceId::new(1);
    let pin = ChunkCoord3 { x: 9, y: 0, z: 0 };
    let first = planner
        .replace_source(
            source,
            snapshot(
                0,
                Some(focus(
                    WorldId(7),
                    [0.0; 3],
                    0,
                    1,
                    DemandDistanceOrder::NearestFirst,
                )),
                [pin],
            ),
        )
        .unwrap();
    assert_eq!(first.entered().len(), 2);
    let replacement = planner
        .replace_source(
            source,
            snapshot(
                0,
                Some(focus(
                    WorldId(7),
                    [32.0, 0.0, 0.0],
                    0,
                    0,
                    DemandDistanceOrder::NearestFirst,
                )),
                [],
            ),
        )
        .unwrap();
    assert_eq!(replacement.exited().len(), 2);
    assert_eq!(planner.effective_snapshot().chunks().len(), 1);
    assert!(planner.remove_source(source).unwrap().exited().len() == 1);
    assert!(planner.remove_source(source).unwrap().is_empty());
}

#[test]
fn hysteresis_is_source_local_and_pins_are_explicit() {
    let mut planner = planner(DemandLimits::default());
    let one = DemandSourceId::new(1);
    let two = DemandSourceId::new(2);
    planner
        .replace_source(
            one,
            snapshot(
                0,
                Some(focus(
                    WorldId(7),
                    [0.0; 3],
                    0,
                    1,
                    DemandDistanceOrder::NearestFirst,
                )),
                [ChunkCoord3 { x: 5, y: 0, z: 0 }],
            ),
        )
        .unwrap();
    planner
        .replace_source(
            two,
            snapshot(
                0,
                Some(focus(
                    WorldId(7),
                    [160.0, 0.0, 0.0],
                    0,
                    0,
                    DemandDistanceOrder::NearestFirst,
                )),
                [],
            ),
        )
        .unwrap();
    planner
        .replace_source(
            one,
            snapshot(
                0,
                Some(focus(
                    WorldId(7),
                    [16.0, 0.0, 0.0],
                    0,
                    1,
                    DemandDistanceOrder::NearestFirst,
                )),
                [],
            ),
        )
        .unwrap();
    let chunks = planner.effective_snapshot().chunks();
    assert!(chunks.iter().any(
        |chunk| chunk.chunk_id().coord == ChunkCoord3 { x: 0, y: 0, z: 0 }
            && chunk.class() == DemandClass::Retained
    ));
    assert!(
        !chunks
            .iter()
            .any(|chunk| chunk.chunk_id().coord == ChunkCoord3 { x: 5, y: 0, z: 0 })
    );
    assert!(
        chunks
            .iter()
            .any(|chunk| chunk.chunk_id().coord == ChunkCoord3 { x: 10, y: 0, z: 0 })
    );
}

#[test]
fn merge_precedence_and_priority_are_stable() {
    let mut planner = planner(DemandLimits::default());
    let coord = ChunkCoord3 { x: 0, y: 0, z: 0 };
    planner
        .apply_transaction(transaction([
            DemandSourceChange::Replace {
                source_id: DemandSourceId::new(2),
                snapshot: snapshot(
                    2,
                    Some(focus(
                        WorldId(7),
                        [0.0; 3],
                        0,
                        0,
                        DemandDistanceOrder::NearestFirst,
                    )),
                    [],
                ),
            },
            DemandSourceChange::Replace {
                source_id: DemandSourceId::new(1),
                snapshot: snapshot(1, None, [coord]),
            },
        ]))
        .unwrap();
    let winner = planner
        .effective_snapshot()
        .get(runen_spatial::ChunkId::new(WorldId(7), coord))
        .unwrap();
    assert_eq!(winner.class(), DemandClass::Pinned);
    assert_eq!(winner.best_source_id(), DemandSourceId::new(1));
}

#[test]
fn ordering_uses_the_accepted_equal_distance_directions() {
    for order in [
        DemandDistanceOrder::NearestFirst,
        DemandDistanceOrder::FarthestFirst,
    ] {
        let mut planner = planner(DemandLimits::default());
        planner
            .replace_source(
                DemandSourceId::new(1),
                snapshot(0, Some(focus(WorldId(7), [0.0; 3], 1, 1, order)), []),
            )
            .unwrap();
        let chunks = planner.effective_snapshot().chunks();
        for adjacent in chunks.windows(2) {
            let left = adjacent[0].chunk_id().coord;
            let right = adjacent[1].chunk_id().coord;
            let left_distance = i128::from(left.x) * i128::from(left.x)
                + i128::from(left.y) * i128::from(left.y)
                + i128::from(left.z) * i128::from(left.z);
            let right_distance = i128::from(right.x) * i128::from(right.x)
                + i128::from(right.y) * i128::from(right.y)
                + i128::from(right.z) * i128::from(right.z);
            match order {
                DemandDistanceOrder::NearestFirst => {
                    assert!(left_distance <= right_distance);
                    if left_distance == right_distance {
                        assert!(left <= right);
                    }
                }
                DemandDistanceOrder::FarthestFirst => {
                    assert!(left_distance >= right_distance);
                    if left_distance == right_distance {
                        assert!(left >= right);
                    }
                }
            }
        }
    }
}

#[test]
fn pressure_and_failures_are_atomic() {
    let limits = DemandLimits::try_new(2, 9, 18, 1).unwrap();
    let mut planner = planner(limits);
    planner
        .replace_source(
            DemandSourceId::new(1),
            snapshot(0, None, [ChunkCoord3 { x: 0, y: 0, z: 0 }]),
        )
        .unwrap();
    let before = planner.effective_snapshot().clone();
    assert!(
        planner
            .replace_source(
                DemandSourceId::new(2),
                snapshot(
                    0,
                    None,
                    [
                        ChunkCoord3 { x: 1, y: 0, z: 0 },
                        ChunkCoord3 { x: 2, y: 0, z: 0 }
                    ]
                )
            )
            .is_err()
    );
    assert_eq!(planner.effective_snapshot(), &before);
    let delta = planner
        .replace_limits(DemandLimits::try_new(2, 9, 18, 3).unwrap())
        .unwrap();
    assert!(delta.is_empty());
}

#[test]
fn world_mismatch_and_callback_failure_preserve_state() {
    let mut planner = planner(DemandLimits::default());
    let before = planner.effective_snapshot().clone();
    assert!(
        planner
            .replace_source(
                DemandSourceId::new(1),
                snapshot(
                    0,
                    Some(focus(
                        WorldId(8),
                        [0.0; 3],
                        0,
                        0,
                        DemandDistanceOrder::NearestFirst
                    )),
                    []
                )
            )
            .is_err()
    );
    assert_eq!(planner.effective_snapshot(), &before);
    let transaction = transaction([DemandSourceChange::Replace {
        source_id: DemandSourceId::new(1),
        snapshot: snapshot(
            0,
            Some(focus(
                WorldId(7),
                [0.0; 3],
                0,
                0,
                DemandDistanceOrder::NearestFirst,
            )),
            [],
        ),
    }]);
    let observed = planner
        .apply_transaction_with(transaction, |snapshot, _| snapshot.len())
        .unwrap();
    assert_eq!(observed, 1);
}
