use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId, WorldPosition};
use runen_spatial_demand::{
    DemandFocus, DemandLimits, DemandSourceChange, DemandSourceId, DemandSourceSnapshot,
};
use runen_spatial_streaming::{
    ChunkAvailability, ChunkOperation, ProviderEvent, ProviderEventKind, StreamRequest,
    StreamRequestId, StreamRequestKind, StreamingBudgets, StreamingTick, WorldStreamingConfig,
    WorldStreamingController, WorldStreamingError, WorldStreamingEvent, WorldStreamingEventKind,
};

fn partition() -> GridPartitionConfig {
    GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap()
}

fn controller(load_budget: usize, unload_budget: usize) -> WorldStreamingController {
    let mut config = WorldStreamingConfig::new(WorldId(7), partition(), DemandLimits::default());
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

fn event_request_ids(events: &[WorldStreamingEvent]) -> Vec<Option<StreamRequestId>> {
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
    assert_eq!(event_request_ids(&events), vec![Some(request.request_id)]);
    let record = controller.record(request.chunk_id).unwrap();
    assert_eq!(record.availability(), ChunkAvailability::Absent);
    assert_eq!(record.operation(), ChunkOperation::Idle);
    assert_eq!(record.blocking_failure(), Some(StreamRequestKind::Load));

    let idle_tick = controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();
    assert!(idle_tick.requests.is_empty());

    let retry_event = controller.retry_blocking_failure(request.chunk_id).unwrap();
    assert_eq!(retry_event.kind, WorldStreamingEventKind::LoadQueued);
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

    let retry_event = controller.retry_blocking_failure(load.chunk_id).unwrap();
    assert_eq!(retry_event.kind, WorldStreamingEventKind::UnloadQueued);
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
fn active_load_reversal_finishes_then_queues_unload() {
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
            WorldStreamingEventKind::UnloadQueued,
        ]
    );
    let record = controller.record(load.chunk_id).unwrap();
    assert!(!record.desired());
    assert_eq!(record.availability(), ChunkAvailability::Resident);
    assert_eq!(record.operation(), ChunkOperation::UnloadQueued);
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
fn active_unload_reversal_finishes_then_queues_load() {
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
            WorldStreamingEventKind::LoadQueued,
        ]
    );
    let record = controller.record(load.chunk_id).unwrap();
    assert!(record.desired());
    assert_eq!(record.availability(), ChunkAvailability::Absent);
    assert_eq!(record.operation(), ChunkOperation::LoadQueued);
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
fn queued_reversal_cancels_without_provider_churn() {
    let mut controller = controller(0, 0);
    let first = controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();
    assert!(first.requests.is_empty());
    let chunk_id = ChunkId::new(WorldId(7), ChunkCoord3 { x: 0, y: 0, z: 0 });
    assert_eq!(
        controller.record(chunk_id).unwrap().operation(),
        ChunkOperation::LoadQueued
    );

    let reversed = controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();
    assert!(reversed.requests.is_empty());
    assert!(controller.record(chunk_id).is_none());
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
fn unissued_queue_refreshes_rank_without_operation_churn() {
    let mut controller = controller(0, 0);
    controller
        .tick(focus_with_radius(0.0, 0.0, 0.0, 1))
        .unwrap();
    let center_after_move = ChunkId::new(WorldId(7), ChunkCoord3 { x: 1, y: 0, z: 0 });
    let old_rank = controller.record(center_after_move).unwrap().rank();

    let update = controller
        .tick(focus_with_radius(16.0, 0.0, 0.0, 1))
        .unwrap();
    assert!(update.requests.is_empty());
    assert_eq!(
        controller.record(center_after_move).unwrap().operation(),
        ChunkOperation::LoadQueued
    );
    assert!(controller.record(center_after_move).unwrap().rank() < old_rank);

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
fn effective_pressure_transitions_desired_chunks_without_duplicate_work() {
    let limits = DemandLimits::try_new(2, 1, 2, 1).unwrap();
    let mut config = WorldStreamingConfig::new(WorldId(7), partition(), limits);
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
    assert!(controller.record(first).unwrap().desired());
    assert!(controller.record(second).is_none());

    controller
        .tick(transaction_tick([DemandSourceChange::Remove {
            source_id: DemandSourceId::new(1),
        }]))
        .unwrap();
    assert!(controller.record(first).is_none());
    assert!(controller.record(second).unwrap().desired());
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
