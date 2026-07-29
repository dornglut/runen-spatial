use runen_spatial::{
    ChunkCoord3, ChunkId, GridPartitionConfig, SpatialMathError, WorldId, WorldPosition,
};
use runen_spatial_demand::{
    DemandAxis, DemandClass, DemandDistanceOrder, DemandFocus, DemandLimitKind, DemandLimits,
    DemandSourceChange, DemandSourceId, DemandSourcePriority, DemandSourceSnapshot,
    DemandTransaction, SpatialDemandError, SpatialDemandPlanner,
};
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};

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

fn source(source_id: u64, priority: u32, x: f64) -> DemandSourceChange {
    DemandSourceChange::Replace {
        source_id: DemandSourceId::new(source_id),
        snapshot: snapshot(
            priority,
            Some(focus(
                WorldId(7),
                [x, 0.0, 0.0],
                0,
                0,
                DemandDistanceOrder::NearestFirst,
            )),
            [],
        ),
    }
}

fn coords(planner: &SpatialDemandPlanner) -> Vec<ChunkCoord3> {
    planner
        .effective_snapshot()
        .chunks()
        .iter()
        .map(|chunk| chunk.chunk_id().coord)
        .collect()
}

fn assert_unchanged(
    planner: &SpatialDemandPlanner,
    limits: DemandLimits,
    source_count: usize,
    snapshot: &runen_spatial_demand::EffectiveDemandSnapshot,
) {
    assert_eq!(planner.limits(), limits);
    assert_eq!(planner.source_count(), source_count);
    assert_eq!(planner.effective_snapshot(), snapshot);
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
fn world_mismatch_preserves_state() {
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

#[test]
fn constructors_report_every_structured_contract_error() {
    for (limits, kind) in [
        ((0, 1, 1, 1), DemandLimitKind::Sources),
        ((1, 0, 1, 1), DemandLimitKind::ContributionsPerSource),
        ((1, 1, 0, 1), DemandLimitKind::TotalContributions),
        ((1, 1, 1, 0), DemandLimitKind::EffectiveChunks),
    ] {
        assert_eq!(
            DemandLimits::try_new(limits.0, limits.1, limits.2, limits.3),
            Err(SpatialDemandError::ZeroLimit { limit: kind })
        );
    }
    let position = WorldPosition::try_new(WorldId(7), [0.0; 3]).unwrap();
    assert_eq!(
        DemandFocus::try_new(position, 2, 1, 0, 0, DemandDistanceOrder::NearestFirst),
        Err(SpatialDemandError::RetainRadiusBelowDesired {
            axis: DemandAxis::Horizontal,
            desired: 2,
            retain: 1,
        })
    );
    assert_eq!(
        DemandFocus::try_new(position, 0, 0, 2, 1, DemandDistanceOrder::NearestFirst),
        Err(SpatialDemandError::RetainRadiusBelowDesired {
            axis: DemandAxis::Vertical,
            desired: 2,
            retain: 1,
        })
    );
    assert_eq!(
        DemandSourceSnapshot::try_new(DemandSourcePriority::new(0), None, []),
        Err(SpatialDemandError::EmptySourceSnapshot)
    );
    assert_eq!(
        DemandTransaction::try_new([
            DemandSourceChange::Remove {
                source_id: DemandSourceId::new(3),
            },
            DemandSourceChange::Remove {
                source_id: DemandSourceId::new(3),
            },
        ]),
        Err(SpatialDemandError::DuplicateSourceChange {
            source_id: DemandSourceId::new(3),
        })
    );
}

#[test]
fn source_limit_precedes_changed_source_construction_and_final_count_is_canonical() {
    let limits = DemandLimits::try_new(1, 16, 16, 16).unwrap();
    let mut planner = planner(limits);
    planner
        .apply_transaction(transaction([source(1, 0, 0.0)]))
        .unwrap();
    let before = planner.effective_snapshot().clone();

    let oversized_mismatch = DemandSourceChange::Replace {
        source_id: DemandSourceId::new(2),
        snapshot: snapshot(
            0,
            Some(focus(
                WorldId(8),
                [0.0; 3],
                u32::MAX,
                u32::MAX,
                DemandDistanceOrder::NearestFirst,
            )),
            [],
        ),
    };
    assert_eq!(
        planner.apply_transaction(transaction([oversized_mismatch.clone()])),
        Err(SpatialDemandError::SourceLimitExceeded {
            limit: 1,
            candidate: 2,
        })
    );
    assert_unchanged(&planner, limits, 1, &before);

    let replacement = planner.apply_transaction(transaction([
        DemandSourceChange::Remove {
            source_id: DemandSourceId::new(1),
        },
        source(2, 0, 16.0),
    ]));
    assert!(replacement.is_ok());
    assert_eq!(planner.source_count(), 1);
    assert_eq!(coords(&planner), vec![ChunkCoord3 { x: 1, y: 0, z: 0 }]);
}

#[test]
fn replacement_removal_and_source_local_retention_have_complete_snapshot_semantics() {
    let mut planner = planner(DemandLimits::default());
    let one = DemandSourceId::new(1);
    let two = DemandSourceId::new(2);
    let pin = ChunkCoord3 { x: 9, y: 0, z: 0 };
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
                [pin],
            ),
        )
        .unwrap();
    planner
        .apply_transaction(transaction([source(2, 0, 32.0)]))
        .unwrap();

    let replacement = planner
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
    assert!(
        planner
            .effective_snapshot()
            .chunks()
            .iter()
            .any(
                |chunk| chunk.chunk_id().coord == ChunkCoord3 { x: 0, y: 0, z: 0 }
                    && chunk.class() == DemandClass::Retained
            )
    );
    assert!(!coords(&planner).contains(&pin));
    assert!(!replacement.is_empty());

    let stable = planner
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
    assert!(stable.is_empty());
    planner
        .replace_source(one, snapshot(0, None, [ChunkCoord3 { x: 4, y: 0, z: 0 }]))
        .unwrap();
    assert!(!coords(&planner).contains(&ChunkCoord3 { x: 0, y: 0, z: 0 }));
    assert!(coords(&planner).contains(&ChunkCoord3 { x: 2, y: 0, z: 0 }));
    assert!(coords(&planner).contains(&ChunkCoord3 { x: 4, y: 0, z: 0 }));
    assert_eq!(
        planner
            .remove_source(DemandSourceId::new(99))
            .unwrap()
            .is_empty(),
        true
    );
    planner.remove_source(two).unwrap();
    assert!(coords(&planner).contains(&ChunkCoord3 { x: 4, y: 0, z: 0 }));
    planner.remove_source(one).unwrap();
    assert!(planner.effective_snapshot().is_empty());
}

#[test]
fn merge_order_ranks_permutations_and_replay_are_deterministic() {
    let changes = [source(2, 1, 16.0), source(1, 1, 0.0), source(3, 2, 32.0)];
    let mut first = planner(DemandLimits::default());
    let first_delta = first
        .apply_transaction(transaction(changes.clone()))
        .unwrap();
    let first_snapshot = first.effective_snapshot().clone();
    for ordering in [
        [changes[0].clone(), changes[1].clone(), changes[2].clone()],
        [changes[2].clone(), changes[0].clone(), changes[1].clone()],
        [changes[1].clone(), changes[2].clone(), changes[0].clone()],
    ] {
        let mut replay = planner(DemandLimits::default());
        let delta = replay.apply_transaction(transaction(ordering)).unwrap();
        assert_eq!(replay.effective_snapshot(), &first_snapshot);
        assert_eq!(delta, first_delta);
    }
    let chunks = first.effective_snapshot().chunks();
    assert_eq!(chunks[0].best_source_id(), DemandSourceId::new(3));
    assert_eq!(chunks[1].best_source_id(), DemandSourceId::new(1));
    assert_eq!(chunks[2].best_source_id(), DemandSourceId::new(2));
    for (index, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.rank().get(), index as u32);
    }
    let mut sequence_first = planner(DemandLimits::default());
    let mut replay = planner(DemandLimits::default());
    for changes in [
        [source(1, 0, 0.0)],
        [source(2, 1, 16.0)],
        [source(1, 2, 0.0)],
    ] {
        let expected = sequence_first
            .apply_transaction(transaction(changes.clone()))
            .unwrap();
        let actual = replay.apply_transaction(transaction(changes)).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(
            replay.effective_snapshot(),
            sequence_first.effective_snapshot()
        );
    }
}

#[test]
fn effective_updates_are_class_rank_and_winner_changes_without_membership_churn() {
    let mut planner = planner(DemandLimits::default());
    planner
        .apply_transaction(transaction([source(1, 0, 0.0), source(2, 1, 16.0)]))
        .unwrap();
    let rank_change = planner
        .apply_transaction(transaction([source(1, 2, 0.0)]))
        .unwrap();
    assert!(rank_change.entered().is_empty() && rank_change.exited().is_empty());
    assert_eq!(rank_change.updated().len(), 2);
    assert!(
        rank_change
            .updated()
            .windows(2)
            .all(|pair| pair[0].rank() < pair[1].rank())
    );

    let shared = ChunkCoord3 { x: 0, y: 0, z: 0 };
    let class_change = planner
        .apply_transaction(transaction([DemandSourceChange::Replace {
            source_id: DemandSourceId::new(3),
            snapshot: snapshot(0, None, [shared]),
        }]))
        .unwrap();
    assert!(class_change.entered().is_empty() && class_change.exited().is_empty());
    assert!(
        class_change
            .updated()
            .iter()
            .any(|chunk| chunk.chunk_id().coord == shared && chunk.class() == DemandClass::Pinned)
    );
    let pin_release = planner
        .apply_transaction(transaction([DemandSourceChange::Remove {
            source_id: DemandSourceId::new(3),
        }]))
        .unwrap();
    assert!(pin_release.updated().iter().any(|chunk| {
        chunk.chunk_id().coord == shared && chunk.class() == DemandClass::Desired
    }));

    let winner_change = planner
        .apply_transaction(transaction([source(2, 3, 16.0)]))
        .unwrap();
    assert!(winner_change.entered().is_empty() && winner_change.exited().is_empty());
    assert!(
        winner_change
            .updated()
            .iter()
            .any(|chunk| chunk.best_source_id() == DemandSourceId::new(2))
    );
    let all = [
        winner_change.entered(),
        winner_change.updated(),
        winner_change.exited(),
    ];
    let ids = all
        .into_iter()
        .flatten()
        .map(|chunk| chunk.chunk_id())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), winner_change.updated().len());
}

#[test]
fn pressure_limits_suppress_reenter_and_reject_without_mutation() {
    let limits = DemandLimits::try_new(3, 9, 9, 2).unwrap();
    let mut planner = planner(limits);
    planner
        .apply_transaction(transaction([
            source(1, 1, 0.0),
            source(2, 3, 16.0),
            source(3, 2, 32.0),
        ]))
        .unwrap();
    assert_eq!(
        coords(&planner),
        vec![
            ChunkCoord3 { x: 1, y: 0, z: 0 },
            ChunkCoord3 { x: 2, y: 0, z: 0 }
        ]
    );
    let pressure = planner.effective_snapshot().pressure();
    assert_eq!(
        (
            pressure.candidate_effective_chunks(),
            pressure.selected_effective_chunks(),
            pressure.unique_pinned_effective_chunks(),
            pressure.suppressed_effective_chunks(),
            pressure.total_source_contributions(),
            pressure.source_count(),
        ),
        (3, 2, 0, 1, 3, 3)
    );
    let increase = planner
        .replace_limits(DemandLimits::try_new(3, 9, 9, 3).unwrap())
        .unwrap();
    assert_eq!(increase.entered().len(), 1);
    let decrease = planner
        .replace_limits(DemandLimits::try_new(3, 9, 9, 1).unwrap())
        .unwrap();
    assert_eq!(decrease.exited().len(), 2);
    let before = planner.effective_snapshot().clone();
    let before_limits = planner.limits();
    assert_eq!(
        planner.replace_limits(DemandLimits::try_new(3, 9, 2, 1).unwrap()),
        Err(SpatialDemandError::TotalContributionLimitExceeded {
            limit: 2,
            candidate: 3,
        })
    );
    assert_eq!(planner.limits(), before_limits);
    assert_eq!(planner.effective_snapshot(), &before);
}

#[test]
fn candidate_and_callback_failures_are_atomic_and_callbacks_observe_precommit_state() {
    let mut planner = planner(DemandLimits::default());
    let initial = planner.effective_snapshot().clone();
    let called = Cell::new(false);
    let mismatch = DemandSourceChange::Replace {
        source_id: DemandSourceId::new(1),
        snapshot: snapshot(
            0,
            Some(focus(
                WorldId(8),
                [0.0; 3],
                0,
                0,
                DemandDistanceOrder::NearestFirst,
            )),
            [],
        ),
    };
    assert!(
        planner
            .apply_transaction_with(transaction([mismatch]), |_, _| called.set(true))
            .is_err()
    );
    assert!(!called.get());
    assert_eq!(planner.effective_snapshot(), &initial);

    let transaction = transaction([source(1, 0, 0.0)]);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = planner.apply_transaction_with(transaction, |snapshot, _| {
            assert_eq!(snapshot.len(), 1);
            panic!("preparation failure");
        });
    }));
    assert!(panic.is_err());
    assert_eq!(planner.effective_snapshot(), &initial);
    assert_eq!(planner.source_count(), 0);

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = planner.replace_limits_with(DemandLimits::try_new(1, 1, 1, 1).unwrap(), |_, _| {
            panic!("limit preparation failure");
        });
    }));
    assert!(panic.is_err());
    assert_eq!(planner.effective_snapshot(), &initial);
    assert_eq!(planner.limits(), DemandLimits::default());
}

#[test]
fn checked_focus_boundaries_and_extreme_pins_are_safe() {
    let mut planner = planner(DemandLimits::try_new(1, u32::MAX, u32::MAX, u32::MAX).unwrap());
    let before = planner.effective_snapshot().clone();
    let out_of_range = f64::MAX;
    let overflow = planner.replace_source(
        DemandSourceId::new(1),
        snapshot(
            0,
            Some(focus(
                WorldId(7),
                [out_of_range, 0.0, 0.0],
                0,
                0,
                DemandDistanceOrder::NearestFirst,
            )),
            [],
        ),
    );
    assert!(matches!(
        overflow,
        Err(SpatialDemandError::SpatialMath(
            SpatialMathError::CoordinateOutOfRange {
                operation: "chunk x"
            }
        ))
    ));
    assert_eq!(planner.effective_snapshot(), &before);
    planner
        .replace_source(
            DemandSourceId::new(1),
            snapshot(
                0,
                None,
                [
                    ChunkCoord3 {
                        x: i64::MAX,
                        y: 0,
                        z: 0,
                    },
                    ChunkCoord3 {
                        x: i64::MIN,
                        y: 0,
                        z: 0,
                    },
                ],
            ),
        )
        .unwrap();
    assert_eq!(
        coords(&planner),
        vec![
            ChunkCoord3 {
                x: i64::MIN,
                y: 0,
                z: 0
            },
            ChunkCoord3 {
                x: i64::MAX,
                y: 0,
                z: 0
            }
        ]
    );
    assert!(
        planner
            .effective_snapshot()
            .chunks()
            .iter()
            .all(|chunk| chunk.chunk_id() == ChunkId::new(WorldId(7), chunk.chunk_id().coord))
    );
}
