use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId, WorldPosition};
use runen_spatial_demand::{
    DemandFocus, DemandLimits, DemandSourceChange, DemandSourceId, DemandSourceSnapshot,
};
use runen_spatial_streaming::{
    ChunkAvailability, ChunkOperation, ProviderEvent, ProviderEventKind, StreamRequest,
    StreamRequestId, StreamRequestKind, StreamingBudgets, StreamingCapacity, StreamingTick,
    WorldStreamingConfig, WorldStreamingController, WorldStreamingError, WorldStreamingEvent,
    WorldStreamingEventKind,
};

fn partition() -> GridPartitionConfig {
    GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap()
}

fn controller(load_budget: usize, unload_budget: usize) -> WorldStreamingController {
    controller_with_capacity(
        load_budget,
        unload_budget,
        StreamingCapacity::new(256, 256, 256),
    )
}

fn controller_with_capacity(
    load_budget: usize,
    unload_budget: usize,
    capacity: StreamingCapacity,
) -> WorldStreamingController {
    let mut config = WorldStreamingConfig::new(
        WorldId(7),
        partition(),
        DemandLimits::default(),
        capacity,
    );
    config.budgets = StreamingBudgets {
        max_load_requests_per_tick: load_budget,
        max_unload_requests_per_tick: unload_budget,
    };
    WorldStreamingController::new(config)
}

fn focus(x: f64, y: f64, z: f64) -> StreamingTick {
    focus_with_radius(x, y, z, 1)
}

fn single_focus(x: f64, y: f64, z: f64) -> StreamingTick {
    focus_with_radius(x, y, z, 0)
}

fn focus_with_radius(x: f64, y: f64, z: f64, radius: u32) -> StreamingTick {
    let focus = DemandFocus::try_new(
        WorldPosition::try_new(WorldId(7), [x, y, z]).unwrap(),
        radius,
        radius,
        0,
        0,
    )
    .unwrap();
    let snapshot = DemandSourceSnapshot::try_new(Some(focus), []).unwrap();
    StreamingTick::from_demand_changes([DemandSourceChange::Replace {
        source_id: DemandSourceId::new(0),
        snapshot,
    }])
}

fn source_focus(source_id: u64, x: f64) -> DemandSourceChange {
    let focus = DemandFocus::try_new(
        WorldPosition::try_new(WorldId(7), [x, 0.0, 0.0]).unwrap(),
        0,
        0,
        0,
        0,
    )
    .unwrap();
    DemandSourceChange::Replace {
        source_id: DemandSourceId::new(source_id),
        snapshot: DemandSourceSnapshot::try_new(Some(focus), []).unwrap(),
    }
}

fn transaction_tick(changes: impl IntoIterator<Item = DemandSourceChange>) -> StreamingTick {
    StreamingTick::from_demand_changes(changes)
}

fn provider_event(request: &StreamRequest, kind: ProviderEventKind) -> ProviderEvent {
    ProviderEvent {
        request_id: request.request_id,
        chunk_id: request.chunk_id,
        kind,
    }
}

fn event_kinds(events: &[WorldStreamingEvent]) -> Vec<WorldStreamingEventKind> {
    events.iter().map(|event| event.kind).collect()
}

fn event_request_ids(events: &[WorldStreamingEvent]) -> Vec<StreamRequestId> {
    events.iter().map(|event| event.request_id).collect()
}

fn single_chunk_controller() -> WorldStreamingController {
    controller(1, 1)
}

fn load_single_chunk(controller: &mut WorldStreamingController) -> StreamRequest {
    controller
        .tick(single_focus(0.0, 0.0, 0.0))
        .unwrap()
        .requests[0]
}

fn make_resident(controller: &mut WorldStreamingController) -> StreamRequest {
    let request = load_single_chunk(controller);
    controller
        .accept_provider_event(provider_event(&request, ProviderEventKind::Completed))
        .unwrap();
    request
}

#[test]
fn load_request_keeps_availability_absent_until_completion() {
    let mut controller = controller(1, 0);
    let request = load_single_chunk(&mut controller);
    let record = controller.record(request.chunk_id).unwrap();
    assert_eq!(record.availability(), ChunkAvailability::Absent);
    assert_eq!(record.operation(), ChunkOperation::LoadRequested(request));

    controller
        .accept_provider_event(provider_event(&request, ProviderEventKind::Started))
        .unwrap();
    let record = controller.record(request.chunk_id).unwrap();
    assert_eq!(record.availability(), ChunkAvailability::Absent);
    assert_eq!(record.operation(), ChunkOperation::Loading(request));

    let events = controller
        .accept_provider_event(provider_event(&request, ProviderEventKind::Completed))
        .unwrap();
    assert_eq!(
        event_kinds(&events),
        vec![
            WorldStreamingEventKind::ProviderCompleted,
            WorldStreamingEventKind::Resident,
        ]
    );
    assert_eq!(
        event_request_ids(&events),
        vec![request.request_id, request.request_id]
    );
    let record = controller.record(request.chunk_id).unwrap();
    assert_eq!(record.availability(), ChunkAvailability::Resident);
    assert_eq!(record.operation(), ChunkOperation::Idle);
    assert_eq!(record.blocking_failure(), None);
}

#[test]
fn unload_keeps_residency_until_completion_and_prunes_neutral_record() {
    let mut controller = single_chunk_controller();
    let load = make_resident(&mut controller);

    let unload = controller
        .tick(single_focus(16.0, 0.0, 0.0))
        .unwrap()
        .requests
        .into_iter()
        .find(|request| request.kind == StreamRequestKind::Unload)
        .unwrap();
    let record = controller.record(load.chunk_id).unwrap();
    assert_eq!(record.availability(), ChunkAvailability::Resident);
    assert_eq!(record.operation(), ChunkOperation::UnloadRequested(unload));

    controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Started))
        .unwrap();
    let record = controller.record(load.chunk_id).unwrap();
    assert_eq!(record.availability(), ChunkAvailability::Resident);
    assert_eq!(record.operation(), ChunkOperation::Unloading(unload));

    controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Completed))
        .unwrap();
    assert!(controller.record(load.chunk_id).is_none());
}

#[test]
fn load_failure_is_blocking_only_while_load_is_still_required() {
    let mut controller = single_chunk_controller();
    let request = load_single_chunk(&mut controller);

    let events = controller
        .accept_provider_event(provider_event(&request, ProviderEventKind::Failed))
        .unwrap();
    assert_eq!(event_request_ids(&events), vec![request.request_id]);
    let record = controller.record(request.chunk_id).unwrap();
    assert_eq!(record.availability(), ChunkAvailability::Absent);
    assert_eq!(record.operation(), ChunkOperation::Idle);
    assert_eq!(record.blocking_failure(), Some(StreamRequestKind::Load));

    let idle_tick = controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();
    assert!(idle_tick.requests.is_empty());

    controller.retry_blocking_failure(request.chunk_id).unwrap();
    assert!(controller.record(request.chunk_id).is_none());
    let retry = controller
        .tick(StreamingTick::without_demand_changes())
        .unwrap();
    assert_eq!(retry.requests.len(), 1);
    assert_ne!(retry.requests[0].request_id, request.request_id);

    let second = retry.requests[0];
    controller
        .accept_provider_event(provider_event(&second, ProviderEventKind::Failed))
        .unwrap();
    controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();
    assert!(controller.record(request.chunk_id).is_none());
}

#[test]
fn unload_failure_preserves_residency_and_supports_explicit_retry() {
    let mut controller = single_chunk_controller();
    let load = make_resident(&mut controller);
    let unload = controller
        .tick(single_focus(16.0, 0.0, 0.0))
        .unwrap()
        .requests
        .into_iter()
        .find(|request| request.kind == StreamRequestKind::Unload)
        .unwrap();

    controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Failed))
        .unwrap();
    let record = controller.record(load.chunk_id).unwrap();
    assert_eq!(record.availability(), ChunkAvailability::Resident);
    assert_eq!(record.operation(), ChunkOperation::Idle);
    assert_eq!(record.blocking_failure(), Some(StreamRequestKind::Unload));

    controller.retry_blocking_failure(load.chunk_id).unwrap();
    let retriable = controller.record(load.chunk_id).unwrap();
    assert_eq!(retriable.availability(), ChunkAvailability::Resident);
    assert_eq!(retriable.blocking_failure(), None);
    let retry = controller
        .tick(StreamingTick::without_demand_changes())
        .unwrap();
    assert_eq!(retry.requests.len(), 1);
    assert_eq!(retry.requests[0].kind, StreamRequestKind::Unload);
    assert_ne!(retry.requests[0].request_id, unload.request_id);
}

#[test]
fn unload_failure_is_cleared_when_intent_reverses_to_resident() {
    let mut controller = single_chunk_controller();
    let load = make_resident(&mut controller);
    let unload = controller
        .tick(single_focus(16.0, 0.0, 0.0))
        .unwrap()
        .requests
        .into_iter()
        .find(|request| request.kind == StreamRequestKind::Unload)
        .unwrap();
    controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Failed))
        .unwrap();

    let update = controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();
    assert!(update.requests.is_empty());
    let record = controller.record(load.chunk_id).unwrap();
    assert!(record.desired());
    assert_eq!(record.availability(), ChunkAvailability::Resident);
    assert_eq!(record.operation(), ChunkOperation::Idle);
    assert_eq!(record.blocking_failure(), None);
}

#[test]
fn active_load_reversal_finishes_resident_then_becomes_pending_unload() {
    let mut controller = single_chunk_controller();
    let load = load_single_chunk(&mut controller);
    controller
        .accept_provider_event(provider_event(&load, ProviderEventKind::Started))
        .unwrap();
    controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();

    let events = controller
        .accept_provider_event(provider_event(&load, ProviderEventKind::Completed))
        .unwrap();
    assert_eq!(
        event_kinds(&events),
        vec![
            WorldStreamingEventKind::ProviderCompleted,
            WorldStreamingEventKind::Resident,
        ]
    );
    let record = controller.record(load.chunk_id).unwrap();
    assert!(!record.desired());
    assert_eq!(record.availability(), ChunkAvailability::Resident);
    assert_eq!(record.operation(), ChunkOperation::Idle);

    let next = controller
        .tick(StreamingTick::without_demand_changes())
        .unwrap();
    assert_eq!(next.requests.len(), 1);
    assert_eq!(next.requests[0].kind, StreamRequestKind::Unload);
}

#[test]
fn active_load_failure_after_reversal_needs_no_retry() {
    let mut controller = single_chunk_controller();
    let load = load_single_chunk(&mut controller);
    controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();

    let events = controller
        .accept_provider_event(provider_event(&load, ProviderEventKind::Failed))
        .unwrap();

    assert_eq!(
        event_kinds(&events),
        vec![WorldStreamingEventKind::ProviderFailed]
    );
    assert!(controller.record(load.chunk_id).is_none());
}

#[test]
fn active_unload_reversal_finishes_absent_without_storing_pending_load_state() {
    let mut controller = single_chunk_controller();
    let load = make_resident(&mut controller);
    let unload = controller
        .tick(single_focus(16.0, 0.0, 0.0))
        .unwrap()
        .requests
        .into_iter()
        .find(|request| request.kind == StreamRequestKind::Unload)
        .unwrap();
    controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Started))
        .unwrap();
    controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();

    let events = controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Completed))
        .unwrap();
    assert_eq!(
        event_kinds(&events),
        vec![
            WorldStreamingEventKind::ProviderCompleted,
            WorldStreamingEventKind::Unloaded,
        ]
    );
    assert!(controller.record(load.chunk_id).is_none());

    let next = controller
        .tick(StreamingTick::without_demand_changes())
        .unwrap();
    assert_eq!(next.requests.len(), 1);
    assert_eq!(next.requests[0].kind, StreamRequestKind::Load);
    assert_eq!(next.requests[0].chunk_id, load.chunk_id);
}

#[test]
fn active_unload_failure_after_reversal_needs_no_retry() {
    let mut controller = single_chunk_controller();
    let load = make_resident(&mut controller);
    let unload = controller
        .tick(single_focus(16.0, 0.0, 0.0))
        .unwrap()
        .requests
        .into_iter()
        .find(|request| request.kind == StreamRequestKind::Unload)
        .unwrap();
    controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();

    let events = controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Failed))
        .unwrap();

    assert_eq!(
        event_kinds(&events),
        vec![WorldStreamingEventKind::ProviderFailed]
    );
    let record = controller.record(load.chunk_id).unwrap();
    assert!(record.desired());
    assert_eq!(record.availability(), ChunkAvailability::Resident);
    assert_eq!(record.operation(), ChunkOperation::Idle);
    assert_eq!(record.blocking_failure(), None);
}

#[test]
fn unissued_demand_requires_no_runtime_record_and_reverses_without_churn() {
    let mut controller = controller(0, 0);
    let first = controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();
    assert!(first.requests.is_empty());
    assert_eq!(first.pressure.deferred_loads(), 1);
    assert_eq!(controller.records().count(), 0);

    let reversed = controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();
    assert!(reversed.requests.is_empty());
    assert_eq!(reversed.pressure.deferred_loads(), 1);
    assert_eq!(controller.records().count(), 0);
    assert!(controller.pending_requests().next().is_none());
}

#[test]
fn lifecycle_event_order_is_deterministic_for_identical_reversal_completion() {
    fn run() -> Vec<WorldStreamingEvent> {
        let mut controller = single_chunk_controller();
        let load = load_single_chunk(&mut controller);
        controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();
        controller
            .accept_provider_event(provider_event(&load, ProviderEventKind::Completed))
            .unwrap()
    }

    assert_eq!(run(), run());
}

#[test]
fn stale_and_mismatched_provider_events_are_rejected() {
    let mut controller = single_chunk_controller();
    let request = load_single_chunk(&mut controller);
    let other_chunk = ChunkId::new(WorldId(7), ChunkCoord3 { x: 99, y: 0, z: 0 });
    let mismatch = controller
        .accept_provider_event(ProviderEvent {
            request_id: request.request_id,
            chunk_id: other_chunk,
            kind: ProviderEventKind::Started,
        })
        .unwrap_err();
    assert_eq!(
        mismatch,
        WorldStreamingError::RequestChunkMismatch {
            request_id: request.request_id,
            expected: request.chunk_id,
            actual: other_chunk,
        }
    );

    controller
        .accept_provider_event(provider_event(&request, ProviderEventKind::Completed))
        .unwrap();
    let stale = controller
        .accept_provider_event(provider_event(&request, ProviderEventKind::Started))
        .unwrap_err();
    assert_eq!(
        stale,
        WorldStreamingError::UnknownRequest {
            request_id: request.request_id,
        }
    );
}

#[test]
fn invalid_demand_transaction_leaves_controller_state_unchanged() {
    let mut controller = single_chunk_controller();
    controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();
    let records_before = controller.records().copied().collect::<Vec<_>>();
    let pending_before = controller.pending_requests().copied().collect::<Vec<_>>();

    let focus = DemandFocus::try_new(
        WorldPosition::try_new(WorldId(8), [0.0, 0.0, 0.0]).unwrap(),
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let snapshot = DemandSourceSnapshot::try_new(Some(focus), []).unwrap();
    let error = controller
        .tick(StreamingTick::from_demand_changes([
            DemandSourceChange::Replace {
                source_id: DemandSourceId::new(0),
                snapshot,
            },
        ]))
        .unwrap_err();

    assert!(matches!(
        error,
        WorldStreamingError::SpatialDemand(runen_spatial_demand::SpatialDemandError::SpatialMath(
            runen_spatial::SpatialMathError::WorldMismatch { .. }
        ))
    ));
    assert_eq!(
        controller.records().copied().collect::<Vec<_>>(),
        records_before
    );
    assert_eq!(
        controller.pending_requests().copied().collect::<Vec<_>>(),
        pending_before
    );
}

#[test]
fn planner_rank_remains_the_unissued_load_order_authority() {
    let mut controller = controller(0, 0);
    controller.tick(focus(0.0, 0.0, 0.0)).unwrap();
    assert_eq!(controller.records().count(), 0);

    controller
        .tick(focus_with_radius(16.0, 0.0, 0.0, 1))
        .unwrap();
    let center_after_move = ChunkId::new(WorldId(7), ChunkCoord3 { x: 1, y: 0, z: 0 });
    assert_eq!(
        controller.effective_demand().chunks()[0].chunk_id(),
        center_after_move
    );

    controller.set_budgets(StreamingBudgets {
        max_load_requests_per_tick: 1,
        max_unload_requests_per_tick: 0,
    });
    let issued = controller
        .tick(StreamingTick::without_demand_changes())
        .unwrap();
    assert_eq!(issued.requests[0].chunk_id, center_after_move);
}

#[test]
fn overlapping_source_removal_preserves_in_flight_request() {
    let mut controller = controller(1, 0);
    let first = controller
        .tick(transaction_tick([
            source_focus(1, 0.0),
            source_focus(2, 0.0),
        ]))
        .unwrap()
        .requests[0];
    let update = controller
        .tick(transaction_tick([DemandSourceChange::Remove {
            source_id: DemandSourceId::new(1),
        }]))
        .unwrap();
    assert!(update.requests.is_empty());
    assert!(update.events.is_empty());
    assert_eq!(
        controller.pending_requests().copied().collect::<Vec<_>>(),
        vec![first]
    );
    assert_eq!(
        controller.record(first.chunk_id).unwrap().operation(),
        ChunkOperation::LoadRequested(first)
    );
}

#[test]
fn effective_demand_pressure_changes_intent_without_materializing_unissued_records() {
    let limits = DemandLimits::try_new(2, 1, 2, 1).unwrap();
    let capacity = StreamingCapacity::new(4, 1, 1);
    let mut config = WorldStreamingConfig::new(WorldId(7), partition(), limits, capacity);
    config.budgets = StreamingBudgets {
        max_load_requests_per_tick: 0,
        max_unload_requests_per_tick: 0,
    };
    let mut controller = WorldStreamingController::new(config);
    controller
        .tick(transaction_tick([
            source_focus(2, 16.0),
            source_focus(1, 0.0),
        ]))
        .unwrap();
    let first = ChunkId::new(WorldId(7), ChunkCoord3 { x: 0, y: 0, z: 0 });
    let second = ChunkId::new(WorldId(7), ChunkCoord3 { x: 1, y: 0, z: 0 });
    assert_eq!(controller.effective_demand().chunks()[0].chunk_id(), first);
    assert_eq!(controller.records().count(), 0);

    controller
        .tick(transaction_tick([DemandSourceChange::Remove {
            source_id: DemandSourceId::new(1),
        }]))
        .unwrap();
    assert_eq!(controller.effective_demand().chunks()[0].chunk_id(), second);
    assert_eq!(controller.records().count(), 0);
}

#[test]
fn invalid_duplicate_demand_batch_leaves_controller_unchanged() {
    let mut controller = controller(0, 0);
    controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();
    let records_before = controller.records().copied().collect::<Vec<_>>();
    let snapshot_before = controller.effective_demand().clone();

    let error = controller
        .tick(transaction_tick([
            DemandSourceChange::Remove {
                source_id: DemandSourceId::new(0),
            },
            DemandSourceChange::Remove {
                source_id: DemandSourceId::new(0),
            },
        ]))
        .unwrap_err();
    assert!(matches!(
        error,
        WorldStreamingError::SpatialDemand(
            runen_spatial_demand::SpatialDemandError::DuplicateSourceChange { .. }
        )
    ));
    assert_eq!(
        controller.records().copied().collect::<Vec<_>>(),
        records_before
    );
    assert_eq!(controller.effective_demand(), &snapshot_before);
}

#[test]
fn tracked_record_capacity_defers_new_loads_without_losing_demand() {
    let capacity = StreamingCapacity::new(1, 1, 1);
    let mut controller = controller_with_capacity(1, 1, capacity);
    let first_load = make_resident(&mut controller);

    let moved = controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();
    assert_eq!(moved.requests.len(), 1);
    assert_eq!(moved.requests[0].kind, StreamRequestKind::Unload);
    assert_eq!(moved.requests[0].chunk_id, first_load.chunk_id);
    assert_eq!(moved.pressure.tracked_records(), 1);
    assert_eq!(moved.pressure.max_tracked_records(), 1);
    assert_eq!(moved.pressure.deferred_loads(), 1);

    for step in 2..=12 {
        let stalled = controller
            .tick(single_focus(f64::from(step * 16), 0.0, 0.0))
            .unwrap();
        assert!(stalled.requests.is_empty());
        assert_eq!(stalled.pressure.tracked_records(), 1);
        assert_eq!(stalled.pressure.deferred_loads(), 1);
        assert_eq!(controller.records().count(), 1);
        assert_eq!(controller.pending_requests().count(), 1);
    }

    let unload = moved.requests[0];
    controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Completed))
        .unwrap();
    assert_eq!(controller.records().count(), 0);

    let resumed = controller
        .tick(StreamingTick::without_demand_changes())
        .unwrap();
    assert_eq!(resumed.requests.len(), 1);
    assert_eq!(resumed.requests[0].kind, StreamRequestKind::Load);
    assert_eq!(
        resumed.requests[0].chunk_id,
        ChunkId::new(WorldId(7), ChunkCoord3 { x: 12, y: 0, z: 0 })
    );
}

#[test]
fn in_flight_load_capacity_blocks_accumulation_across_ticks() {
    let capacity = StreamingCapacity::new(16, 1, 4);
    let mut controller = controller_with_capacity(4, 4, capacity);

    let first = controller.tick(focus(0.0, 0.0, 0.0)).unwrap();
    assert_eq!(first.requests.len(), 1);
    assert_eq!(first.pressure.in_flight_loads(), 1);
    assert_eq!(first.pressure.max_in_flight_loads(), 1);
    assert!(first.pressure.deferred_loads() > 0);

    let second = controller
        .tick(StreamingTick::without_demand_changes())
        .unwrap();
    assert!(second.requests.is_empty());
    assert_eq!(second.pressure.in_flight_loads(), 1);
    assert_eq!(controller.pending_requests().count(), 1);
    assert_eq!(controller.records().count(), 1);
}

#[test]
fn saturated_load_capacity_does_not_starve_unload_issuance() {
    let capacity = StreamingCapacity::new(2, 1, 1);
    let mut controller = controller_with_capacity(4, 4, capacity);
    let resident = make_resident(&mut controller);

    let moved = controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();
    assert_eq!(moved.requests.len(), 2);
    assert!(
        moved
            .requests
            .iter()
            .any(|request| request.kind == StreamRequestKind::Load)
    );
    assert!(moved.requests.iter().any(|request| {
        request.kind == StreamRequestKind::Unload && request.chunk_id == resident.chunk_id
    }));
    assert_eq!(moved.pressure.in_flight_loads(), 1);
    assert_eq!(moved.pressure.in_flight_unloads(), 1);
}

#[test]
fn in_flight_unload_capacity_blocks_accumulation() {
    let capacity = StreamingCapacity::new(2, 2, 1);
    let mut controller = controller_with_capacity(2, 2, capacity);
    let loads = controller.tick(focus(0.0, 0.0, 0.0)).unwrap().requests;
    assert_eq!(loads.len(), 2);
    for load in &loads {
        controller
            .accept_provider_event(provider_event(load, ProviderEventKind::Completed))
            .unwrap();
    }
    assert_eq!(controller.records().count(), 2);

    let moved = controller.tick(single_focus(160.0, 0.0, 0.0)).unwrap();
    assert_eq!(
        moved
            .requests
            .iter()
            .filter(|request| request.kind == StreamRequestKind::Unload)
            .count(),
        1
    );
    assert_eq!(moved.pressure.in_flight_unloads(), 1);
    assert_eq!(moved.pressure.max_in_flight_unloads(), 1);
    assert_eq!(moved.pressure.remaining_unloads(), 1);

    let stalled = controller
        .tick(StreamingTick::without_demand_changes())
        .unwrap();
    assert!(stalled.requests.is_empty());
    assert_eq!(stalled.pressure.in_flight_unloads(), 1);
    assert_eq!(stalled.pressure.remaining_unloads(), 1);
}

#[test]
fn unload_order_uses_rank_then_chunk_identity() {
    let capacity = StreamingCapacity::new(3, 3, 3);
    let mut controller = controller_with_capacity(3, 0, capacity);
    let loads = controller.tick(focus(0.0, 0.0, 0.0)).unwrap().requests;
    assert_eq!(loads.len(), 3);
    for load in &loads {
        controller
            .accept_provider_event(provider_event(load, ProviderEventKind::Completed))
            .unwrap();
    }

    controller.tick(single_focus(160.0, 0.0, 0.0)).unwrap();
    let mut expected = controller
        .records()
        .map(|record| (record.rank(), record.chunk_id()))
        .collect::<Vec<_>>();
    expected.sort();

    controller.set_budgets(StreamingBudgets {
        max_load_requests_per_tick: 0,
        max_unload_requests_per_tick: 3,
    });
    let output = controller
        .tick(StreamingTick::without_demand_changes())
        .unwrap();
    let actual = output
        .requests
        .iter()
        .map(|request| (request.rank, request.chunk_id))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn per_tick_budget_remains_independent_from_in_flight_capacity() {
    let capacity = StreamingCapacity::new(16, 8, 8);
    let mut controller = controller_with_capacity(1, 1, capacity);
    let output = controller.tick(focus(0.0, 0.0, 0.0)).unwrap();
    assert_eq!(output.requests.len(), 1);
    assert_eq!(output.pressure.in_flight_loads(), 1);
    assert!(output.pressure.deferred_loads() > 0);
}

#[test]
fn zero_capacity_is_explicit_and_does_not_materialize_waiting_loads() {
    let capacity = StreamingCapacity::new(0, 0, 0);
    let mut controller = controller_with_capacity(8, 8, capacity);
    let output = controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();

    assert!(output.requests.is_empty());
    assert_eq!(output.pressure.tracked_records(), 0);
    assert_eq!(output.pressure.max_tracked_records(), 0);
    assert_eq!(output.pressure.deferred_loads(), 1);
    assert_eq!(controller.records().count(), 0);
    assert_eq!(controller.pending_requests().count(), 0);
}
