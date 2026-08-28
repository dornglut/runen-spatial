use runen_spatial::{
    ChunkCoord3, ChunkId, GridPartitionConfig, SpatialMathError, WorldId, WorldPosition,
};
use runen_spatial_demand::{
    DemandAxis, DemandClass, DemandFocus, DemandLimitKind, DemandLimits, DemandSourceChange,
    DemandSourceId, DemandSourceSnapshot, SpatialDemandError, SpatialDemandPlanner,
};

fn partition() -> GridPartitionConfig {
    GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap()
}

fn planner(limits: DemandLimits) -> SpatialDemandPlanner {
    SpatialDemandPlanner::new(WorldId(7), partition(), limits)
}

fn focus(
    world: WorldId,
    meters: [f64; 3],
    horizontal_desired: u32,
    horizontal_retain: u32,
    vertical_desired: u32,
    vertical_retain: u32,
) -> DemandFocus {
    DemandFocus::try_new(
        WorldPosition::try_new(world, meters).unwrap(),
        horizontal_desired,
        horizontal_retain,
        vertical_desired,
        vertical_retain,
    )
    .unwrap()
}

fn snapshot(
    focus: Option<DemandFocus>,
    pins: impl IntoIterator<Item = ChunkId>,
) -> DemandSourceSnapshot {
    DemandSourceSnapshot::try_new(focus, pins).unwrap()
}

fn replace(source: u64, focus: DemandFocus) -> DemandSourceChange {
    DemandSourceChange::Replace {
        source_id: DemandSourceId::new(source),
        snapshot: snapshot(Some(focus), []),
    }
}

#[test]
fn constructors_reject_invalid_contracts() {
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
        DemandFocus::try_new(position, 2, 1, 0, 0),
        Err(SpatialDemandError::RetainRadiusBelowDesired {
            axis: DemandAxis::Horizontal,
            desired: 2,
            retain: 1,
        })
    );
    assert_eq!(
        DemandFocus::try_new(position, 0, 0, 2, 1),
        Err(SpatialDemandError::RetainRadiusBelowDesired {
            axis: DemandAxis::Vertical,
            desired: 2,
            retain: 1,
        })
    );
    assert_eq!(
        DemandSourceSnapshot::try_new(None, []),
        Err(SpatialDemandError::EmptySourceSnapshot)
    );
}

#[test]
fn complete_replacement_and_removal_are_atomic() {
    let mut planner = planner(DemandLimits::default());
    let source = DemandSourceId::new(1);
    planner
        .replace_source(
            source,
            snapshot(Some(focus(WorldId(7), [0.0; 3], 0, 1, 0, 0)), []),
        )
        .unwrap();
    let replacement = planner
        .replace_source(
            source,
            snapshot(Some(focus(WorldId(7), [32.0, 0.0, 0.0], 0, 0, 0, 0)), []),
        )
        .unwrap();
    assert_eq!(replacement.entered().len(), 1);
    assert_eq!(replacement.exited().len(), 1);
    assert_eq!(planner.effective_snapshot().len(), 1);
    assert_eq!(planner.remove_source(source).unwrap().exited().len(), 1);
    assert!(planner.remove_source(source).unwrap().is_empty());
}

#[test]
fn source_local_hysteresis_does_not_leak_between_sources() {
    let mut planner = planner(DemandLimits::default());
    planner
        .apply_changes([
            replace(1, focus(WorldId(7), [0.0; 3], 0, 1, 0, 0)),
            replace(2, focus(WorldId(7), [160.0, 0.0, 0.0], 0, 0, 0, 0)),
        ])
        .unwrap();
    planner
        .replace_source(
            DemandSourceId::new(1),
            snapshot(Some(focus(WorldId(7), [16.0, 0.0, 0.0], 0, 1, 0, 0)), []),
        )
        .unwrap();
    let chunks = planner.effective_snapshot().chunks();
    assert!(chunks.iter().any(|chunk| {
        chunk.chunk_id().coord == ChunkCoord3 { x: 0, y: 0, z: 0 }
            && chunk.class() == DemandClass::Retained
    }));
    assert!(chunks.iter().any(|chunk| {
        chunk.chunk_id().coord == ChunkCoord3 { x: 10, y: 0, z: 0 }
            && chunk.class() == DemandClass::Desired
    }));
}

#[test]
fn world_mismatch_for_focus_or_pin_preserves_state() {
    let mut planner = planner(DemandLimits::default());
    planner
        .replace_source(
            DemandSourceId::new(1),
            snapshot(Some(focus(WorldId(7), [0.0; 3], 0, 0, 0, 0)), []),
        )
        .unwrap();
    let before = planner.effective_snapshot().clone();

    let wrong_focus = planner.replace_source(
        DemandSourceId::new(1),
        snapshot(Some(focus(WorldId(8), [0.0; 3], 0, 0, 0, 0)), []),
    );
    assert!(matches!(
        wrong_focus,
        Err(SpatialDemandError::SpatialMath(
            SpatialMathError::WorldMismatch { .. }
        ))
    ));
    assert_eq!(planner.effective_snapshot(), &before);

    let wrong_pin = planner.replace_source(
        DemandSourceId::new(2),
        snapshot(
            None,
            [ChunkId::new(WorldId(8), ChunkCoord3 { x: 0, y: 0, z: 0 })],
        ),
    );
    assert!(matches!(
        wrong_pin,
        Err(SpatialDemandError::SpatialMath(
            SpatialMathError::WorldMismatch { .. }
        ))
    ));
    assert_eq!(planner.effective_snapshot(), &before);
}

#[test]
fn desired_volume_is_rejected_before_materialization() {
    let limits = DemandLimits::try_new(1, 100, 100, 100).unwrap();
    let mut planner = planner(limits);
    assert_eq!(
        planner.replace_source(
            DemandSourceId::new(1),
            snapshot(Some(focus(WorldId(7), [0.0; 3], 10, 10, 10, 10)), []),
        ),
        Err(SpatialDemandError::PerSourceContributionLimitExceeded {
            source_id: DemandSourceId::new(1),
            limit: 100,
            candidate: 9_261,
        })
    );
    assert!(planner.effective_snapshot().is_empty());
}

#[test]
fn pins_override_focus_and_cannot_be_suppressed() {
    let limits = DemandLimits::try_new(1, 4, 4, 1).unwrap();
    let mut planner = planner(limits);
    let pinned = ChunkId::new(WorldId(7), ChunkCoord3 { x: 9, y: 0, z: 0 });
    planner
        .replace_source(
            DemandSourceId::new(1),
            snapshot(Some(focus(WorldId(7), [0.0; 3], 0, 0, 0, 0)), [pinned]),
        )
        .unwrap();
    assert_eq!(planner.effective_snapshot().chunks()[0].chunk_id(), pinned);
    assert_eq!(
        planner.effective_snapshot().chunks()[0].class(),
        DemandClass::Pinned
    );

    let before = planner.effective_snapshot().clone();
    let second_pin = ChunkId::new(WorldId(7), ChunkCoord3 { x: 10, y: 0, z: 0 });
    assert_eq!(
        planner.replace_source(DemandSourceId::new(1), snapshot(None, [pinned, second_pin]),),
        Err(SpatialDemandError::PinnedCapacityExceeded {
            limit: 1,
            pinned: 2
        })
    );
    assert_eq!(planner.effective_snapshot(), &before);
}

#[test]
fn equal_sources_interleave_by_local_rank_before_source_id() {
    let limits = DemandLimits::try_new(2, 3, 6, 4).unwrap();
    let mut planner = planner(limits);
    planner
        .apply_changes([
            replace(2, focus(WorldId(7), [160.0, 0.0, 0.0], 0, 0, 1, 1)),
            replace(1, focus(WorldId(7), [0.0, 0.0, 0.0], 0, 0, 1, 1)),
        ])
        .unwrap();
    let coords = planner
        .effective_snapshot()
        .chunks()
        .iter()
        .map(|chunk| chunk.chunk_id().coord)
        .collect::<Vec<_>>();
    assert_eq!(
        coords,
        vec![
            ChunkCoord3 { x: 0, y: 0, z: 0 },
            ChunkCoord3 { x: 10, y: 0, z: 0 },
            ChunkCoord3 { x: 0, y: -1, z: 0 },
            ChunkCoord3 { x: 10, y: -1, z: 0 },
        ]
    );
    assert_eq!(
        planner
            .effective_snapshot()
            .pressure()
            .suppressed_effective_chunks(),
        2
    );
}

#[test]
fn suppressed_demand_reenters_after_higher_ranked_source_is_removed() {
    let limits = DemandLimits::try_new(2, 1, 2, 1).unwrap();
    let mut planner = planner(limits);
    planner
        .apply_changes([
            replace(2, focus(WorldId(7), [16.0, 0.0, 0.0], 0, 0, 0, 0)),
            replace(1, focus(WorldId(7), [0.0; 3], 0, 0, 0, 0)),
        ])
        .unwrap();
    assert_eq!(
        planner.effective_snapshot().chunks()[0].chunk_id().coord,
        ChunkCoord3 { x: 0, y: 0, z: 0 }
    );
    assert_eq!(
        planner
            .effective_snapshot()
            .pressure()
            .suppressed_effective_chunks(),
        1
    );

    let delta = planner.remove_source(DemandSourceId::new(1)).unwrap();
    assert_eq!(delta.entered().len(), 1);
    assert_eq!(delta.exited().len(), 1);
    assert_eq!(
        planner.effective_snapshot().chunks()[0].chunk_id().coord,
        ChunkCoord3 { x: 1, y: 0, z: 0 }
    );
}

#[test]
fn duplicate_change_and_source_limit_fail_atomically() {
    let limits = DemandLimits::try_new(1, 1, 1, 1).unwrap();
    let mut planner = planner(limits);
    planner
        .apply_changes([replace(1, focus(WorldId(7), [0.0; 3], 0, 0, 0, 0))])
        .unwrap();
    let before = planner.effective_snapshot().clone();

    assert_eq!(
        planner.apply_changes([
            DemandSourceChange::Remove {
                source_id: DemandSourceId::new(1)
            },
            DemandSourceChange::Remove {
                source_id: DemandSourceId::new(1)
            },
        ]),
        Err(SpatialDemandError::DuplicateSourceChange {
            source_id: DemandSourceId::new(1),
        })
    );
    assert_eq!(planner.effective_snapshot(), &before);

    assert_eq!(
        planner.apply_changes([replace(2, focus(WorldId(8), [0.0; 3], 0, 0, 0, 0))]),
        Err(SpatialDemandError::SourceLimitExceeded {
            limit: 1,
            candidate: 2
        })
    );
    assert_eq!(planner.effective_snapshot(), &before);
}

#[test]
fn input_permutation_and_replay_are_deterministic() {
    let changes_a = [
        replace(2, focus(WorldId(7), [32.0, 0.0, 0.0], 0, 0, 1, 1)),
        replace(1, focus(WorldId(7), [0.0; 3], 0, 0, 1, 1)),
    ];
    let changes_b = [changes_a[1].clone(), changes_a[0].clone()];
    let mut first = planner(DemandLimits::default());
    let mut second = planner(DemandLimits::default());
    first.apply_changes(changes_a.clone()).unwrap();
    second.apply_changes(changes_b).unwrap();
    assert_eq!(first.effective_snapshot(), second.effective_snapshot());
    let snapshot = first.effective_snapshot().clone();
    assert!(first.apply_changes(changes_a).unwrap().is_empty());
    assert_eq!(first.effective_snapshot(), &snapshot);
}

#[test]
fn distant_sources_with_bounded_local_radii_do_not_overflow_ranking() {
    let partition = GridPartitionConfig::try_new(1.0, [1, 1, 1]).unwrap();
    let mut planner = SpatialDemandPlanner::new(WorldId(7), partition, DemandLimits::default());
    let far = 2_f64.powi(62);
    planner
        .apply_changes([
            replace(1, focus(WorldId(7), [-far, 0.0, 0.0], 0, 0, 1, 1)),
            replace(2, focus(WorldId(7), [far, 0.0, 0.0], 0, 0, 1, 1)),
        ])
        .unwrap();
    assert_eq!(planner.effective_snapshot().len(), 6);
}

#[test]
fn total_contribution_limit_rejects_the_batch_atomically() {
    let limits = DemandLimits::try_new(2, 25, 40, 40).unwrap();
    let mut planner = planner(limits);

    let error = planner
        .apply_changes([
            replace(1, focus(WorldId(7), [0.0; 3], 2, 2, 0, 0)),
            replace(2, focus(WorldId(7), [160.0, 0.0, 0.0], 2, 2, 0, 0)),
        ])
        .unwrap_err();

    assert_eq!(
        error,
        SpatialDemandError::TotalContributionLimitExceeded {
            limit: 40,
            candidate: 50,
        }
    );
    assert_eq!(planner.source_count(), 0);
    assert!(planner.effective_snapshot().is_empty());
}

#[test]
fn delta_distinguishes_entry_class_change_rerank_and_exit() {
    let mut planner = planner(DemandLimits::default());
    let source = DemandSourceId::new(1);
    planner
        .replace_source(
            source,
            snapshot(Some(focus(WorldId(7), [0.0; 3], 0, 1, 0, 0)), []),
        )
        .unwrap();

    let moved_once = planner
        .replace_source(
            source,
            snapshot(Some(focus(WorldId(7), [16.0, 0.0, 0.0], 0, 1, 0, 0)), []),
        )
        .unwrap();
    assert_eq!(moved_once.entered().len(), 1);
    assert_eq!(moved_once.updated().len(), 1);
    assert!(moved_once.exited().is_empty());
    assert_eq!(
        moved_once.updated()[0].chunk_id().coord,
        ChunkCoord3 { x: 0, y: 0, z: 0 }
    );
    assert_eq!(moved_once.updated()[0].class(), DemandClass::Retained);

    let moved_twice = planner
        .replace_source(
            source,
            snapshot(Some(focus(WorldId(7), [32.0, 0.0, 0.0], 0, 1, 0, 0)), []),
        )
        .unwrap();
    assert_eq!(moved_twice.entered().len(), 1);
    assert_eq!(moved_twice.updated().len(), 1);
    assert_eq!(moved_twice.exited().len(), 1);
    assert_eq!(
        moved_twice.exited()[0].chunk_id().coord,
        ChunkCoord3 { x: 0, y: 0, z: 0 }
    );
}
