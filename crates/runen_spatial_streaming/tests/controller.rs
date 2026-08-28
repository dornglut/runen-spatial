use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId, WorldPosition};
use runen_spatial_demand::{
    DemandFocus, DemandLimits, DemandSourceChange, DemandSourceId, DemandSourceSnapshot,
};
use runen_spatial_streaming::{
    ChunkLifecycleState, ProviderEvent, ProviderEventKind, StreamRequest, StreamRequestId,
    StreamRequestKind, StreamingBudgets, StreamingTick, WorldStreamingConfig,
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
    let mut config = WorldStreamingConfig::new(WorldId(7), partition(), DemandLimits::default());
    config.budgets = StreamingBudgets {
        max_load_requests_per_tick: 1,
        max_unload_requests_per_tick: 1,
    };
    WorldStreamingController::new(config)
}

#[test]
fn tick_emits_budgeted_load_requests_without_loading_payloads() {
    let mut controller = controller(2, 4);

    let output = controller.tick(focus(0.0, 0.0, 0.0)).unwrap();

    assert_eq!(output.requests.len(), 2);
    assert!(
        output
            .requests
            .iter()
            .all(|request| request.kind == StreamRequestKind::Load)
    );
    assert_eq!(
        output.requests[0].chunk_id,
        ChunkId::new(WorldId(7), ChunkCoord3 { x: 0, y: 0, z: 0 })
    );
    assert_eq!(
        controller
            .record(output.requests[0].chunk_id)
            .map(|record| record.state),
        Some(ChunkLifecycleState::LoadRequested)
    );
}

#[test]
fn provider_started_and_completed_advance_to_resident() {
    let mut controller = controller(1, 4);
    let output = controller.tick(focus(0.0, 0.0, 0.0)).unwrap();
    let request = output.requests[0];

    let started = controller
        .accept_provider_event(provider_event(&request, ProviderEventKind::Started))
        .unwrap();
    assert_eq!(started[0].kind, WorldStreamingEventKind::ProviderStarted);
    assert_eq!(started[0].request_id, Some(request.request_id));
    assert_eq!(
        controller
            .record(request.chunk_id)
            .map(|record| record.state),
        Some(ChunkLifecycleState::Loading)
    );

    let completed = controller
        .accept_provider_event(provider_event(&request, ProviderEventKind::Completed))
        .unwrap();
    assert!(
        completed
            .iter()
            .any(|event| event.kind == WorldStreamingEventKind::Resident)
    );
    assert_eq!(
        event_request_ids(&completed),
        vec![Some(request.request_id), Some(request.request_id)]
    );
    assert_eq!(
        controller
            .record(request.chunk_id)
            .map(|record| record.state),
        Some(ChunkLifecycleState::Resident)
    );
}

#[test]
fn resident_chunk_exiting_desired_set_queues_unload_then_unloads() {
    let mut config = WorldStreamingConfig::new(WorldId(7), partition(), DemandLimits::default());
    config.budgets = StreamingBudgets {
        max_load_requests_per_tick: 1,
        max_unload_requests_per_tick: 1,
    };
    let mut controller = WorldStreamingController::new(config);

    let load = controller
        .tick(single_focus(0.0, 0.0, 0.0))
        .unwrap()
        .requests[0];
    controller
        .accept_provider_event(provider_event(&load, ProviderEventKind::Completed))
        .unwrap();

    let unload_tick = controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();
    let unload = unload_tick
        .requests
        .iter()
        .find(|request| request.kind == StreamRequestKind::Unload)
        .copied()
        .expect("resident chunk should request unload");

    assert_eq!(unload.chunk_id, load.chunk_id);
    assert_eq!(
        controller.record(load.chunk_id).map(|record| record.state),
        Some(ChunkLifecycleState::UnloadRequested)
    );

    controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Started))
        .unwrap();
    controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Completed))
        .unwrap();
    assert_eq!(
        controller.record(load.chunk_id).map(|record| record.state),
        Some(ChunkLifecycleState::Absent)
    );
}

#[test]
fn provider_failure_does_not_auto_retry_when_still_desired() {
    let mut controller = single_chunk_controller();
    let first = controller
        .tick(single_focus(0.0, 0.0, 0.0))
        .unwrap()
        .requests[0];

    let events = controller
        .accept_provider_event(provider_event(&first, ProviderEventKind::Failed))
        .unwrap();
    assert_eq!(event_request_ids(&events), vec![Some(first.request_id)]);
    assert_eq!(
        controller.record(first.chunk_id).map(|record| record.state),
        Some(ChunkLifecycleState::Failed)
    );

    let retry = controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();
    assert!(retry.requests.is_empty());
}

#[test]
fn explicit_retry_failed_chunk_queues_load_when_still_desired() {
    let mut controller = controller(1, 4);
    let first = controller.tick(focus(0.0, 0.0, 0.0)).unwrap().requests[0];

    controller
        .accept_provider_event(provider_event(&first, ProviderEventKind::Failed))
        .unwrap();

    let event = controller.retry_failed_chunk(first.chunk_id).unwrap();
    assert_eq!(event.kind, WorldStreamingEventKind::LoadQueued);
    assert_eq!(event.request_id, None);

    let retry = controller.tick(focus(0.0, 0.0, 0.0)).unwrap();
    assert_eq!(retry.requests.len(), 1);
    assert_eq!(retry.requests[0].chunk_id, first.chunk_id);
    assert_eq!(retry.requests[0].kind, StreamRequestKind::Load);
    assert_ne!(retry.requests[0].request_id, first.request_id);
}

#[test]
fn resident_chunk_can_fail_without_payload_ownership() {
    let mut controller = controller(1, 4);
    let request = controller.tick(focus(0.0, 0.0, 0.0)).unwrap().requests[0];
    controller
        .accept_provider_event(provider_event(&request, ProviderEventKind::Completed))
        .unwrap();

    let event = controller.fail_resident_chunk(request.chunk_id).unwrap();

    assert_eq!(event.kind, WorldStreamingEventKind::ProviderFailed);
    assert_eq!(event.request_id, None);
    assert_eq!(
        controller
            .record(request.chunk_id)
            .map(|record| record.state),
        Some(ChunkLifecycleState::Failed)
    );
}

#[test]
fn load_request_becoming_undesired_before_provider_starts_queues_unload_after_completion() {
    let mut controller = single_chunk_controller();
    let load = controller
        .tick(single_focus(0.0, 0.0, 0.0))
        .unwrap()
        .requests[0];

    controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();
    let record = controller.record(load.chunk_id).unwrap();
    assert_eq!(record.state, ChunkLifecycleState::LoadRequested);
    assert!(!record.desired);

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
    assert_eq!(
        event_request_ids(&events),
        vec![Some(load.request_id), Some(load.request_id), None]
    );
    assert_eq!(
        controller.record(load.chunk_id).map(|record| record.state),
        Some(ChunkLifecycleState::UnloadQueued)
    );
}

#[test]
fn load_request_becoming_undesired_while_loading_queues_unload_after_completion() {
    let mut controller = single_chunk_controller();
    let load = controller
        .tick(single_focus(0.0, 0.0, 0.0))
        .unwrap()
        .requests[0];
    controller
        .accept_provider_event(provider_event(&load, ProviderEventKind::Started))
        .unwrap();

    controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();
    let record = controller.record(load.chunk_id).unwrap();
    assert_eq!(record.state, ChunkLifecycleState::Loading);
    assert!(!record.desired);

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
    assert_eq!(
        controller.record(load.chunk_id).map(|record| record.state),
        Some(ChunkLifecycleState::UnloadQueued)
    );
}

#[test]
fn unload_request_becoming_desired_before_provider_starts_queues_load_after_unload_completion() {
    let mut controller = single_chunk_controller();
    let load = controller
        .tick(single_focus(0.0, 0.0, 0.0))
        .unwrap()
        .requests[0];
    controller
        .accept_provider_event(provider_event(&load, ProviderEventKind::Completed))
        .unwrap();
    controller.set_budgets(StreamingBudgets {
        max_load_requests_per_tick: 0,
        max_unload_requests_per_tick: 1,
    });

    let unload = controller
        .tick(single_focus(16.0, 0.0, 0.0))
        .unwrap()
        .requests
        .into_iter()
        .find(|request| request.chunk_id == load.chunk_id)
        .unwrap();
    controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();
    let record = controller.record(load.chunk_id).unwrap();
    assert_eq!(record.state, ChunkLifecycleState::UnloadRequested);
    assert!(record.desired);

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
    assert_eq!(
        event_request_ids(&events),
        vec![Some(unload.request_id), Some(unload.request_id), None]
    );
    assert_eq!(
        controller.record(load.chunk_id).map(|record| record.state),
        Some(ChunkLifecycleState::LoadQueued)
    );
}

#[test]
fn unload_request_becoming_desired_while_unloading_queues_load_after_unload_completion() {
    let mut controller = single_chunk_controller();
    let load = controller
        .tick(single_focus(0.0, 0.0, 0.0))
        .unwrap()
        .requests[0];
    controller
        .accept_provider_event(provider_event(&load, ProviderEventKind::Completed))
        .unwrap();
    controller.set_budgets(StreamingBudgets {
        max_load_requests_per_tick: 0,
        max_unload_requests_per_tick: 1,
    });

    let unload = controller
        .tick(single_focus(16.0, 0.0, 0.0))
        .unwrap()
        .requests
        .into_iter()
        .find(|request| request.chunk_id == load.chunk_id)
        .unwrap();
    controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Started))
        .unwrap();
    controller.tick(single_focus(0.0, 0.0, 0.0)).unwrap();
    let record = controller.record(load.chunk_id).unwrap();
    assert_eq!(record.state, ChunkLifecycleState::Unloading);
    assert!(record.desired);

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
    assert_eq!(
        controller.record(load.chunk_id).map(|record| record.state),
        Some(ChunkLifecycleState::LoadQueued)
    );
}

#[test]
fn lifecycle_event_order_is_deterministic_for_identical_reversal_completion() {
    fn run() -> Vec<WorldStreamingEvent> {
        let mut controller = single_chunk_controller();
        let load = controller
            .tick(single_focus(0.0, 0.0, 0.0))
            .unwrap()
            .requests[0];
        controller.tick(single_focus(16.0, 0.0, 0.0)).unwrap();
        controller
            .accept_provider_event(provider_event(&load, ProviderEventKind::Completed))
            .unwrap()
    }

    assert_eq!(run(), run());
}

#[test]
fn request_order_is_deterministic_for_same_focus() {
    let mut first = controller(9, 4);
    let mut second = controller(9, 4);

    let first_ids = first
        .tick(focus(0.0, 0.0, 0.0))
        .unwrap()
        .requests
        .into_iter()
        .map(|request| request.chunk_id)
        .collect::<Vec<_>>();
    let second_ids = second
        .tick(focus(0.0, 0.0, 0.0))
        .unwrap()
        .requests
        .into_iter()
        .map(|request| request.chunk_id)
        .collect::<Vec<_>>();

    assert_eq!(first_ids, second_ids);
}

#[test]
fn stale_provider_event_after_completion_is_rejected() {
    let mut controller = controller(1, 4);
    let request = controller.tick(focus(0.0, 0.0, 0.0)).unwrap().requests[0];
    controller
        .accept_provider_event(provider_event(&request, ProviderEventKind::Completed))
        .unwrap();

    let error = controller
        .accept_provider_event(provider_event(&request, ProviderEventKind::Started))
        .unwrap_err();

    assert_eq!(
        error,
        WorldStreamingError::UnknownRequest {
            request_id: request.request_id
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
fn unissued_queue_refreshes_rank_without_lifecycle_churn() {
    let mut controller = controller(0, 0);
    controller
        .tick(focus_with_radius(0.0, 0.0, 0.0, 1))
        .unwrap();
    let center_after_move = ChunkId::new(WorldId(7), ChunkCoord3 { x: 1, y: 0, z: 0 });
    let old_rank = controller.record(center_after_move).unwrap().rank;

    let update = controller
        .tick(focus_with_radius(16.0, 0.0, 0.0, 1))
        .unwrap();
    assert!(update.requests.is_empty());
    assert!(update.events.iter().all(|event| {
        event.kind != WorldStreamingEventKind::LoadQueued || event.chunk_id != center_after_move
    }));
    assert_eq!(
        controller.record(center_after_move).unwrap().state,
        ChunkLifecycleState::LoadQueued
    );
    assert!(controller.record(center_after_move).unwrap().rank < old_rank);

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
        controller.record(first.chunk_id).unwrap().state,
        ChunkLifecycleState::LoadRequested
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
    assert!(controller.record(first).unwrap().desired);
    assert!(
        controller
            .record(second)
            .is_none_or(|record| !record.desired)
    );

    controller
        .tick(transaction_tick([DemandSourceChange::Remove {
            source_id: DemandSourceId::new(1),
        }]))
        .unwrap();
    assert!(!controller.record(first).unwrap().desired);
    assert!(controller.record(second).unwrap().desired);
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
