use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
use runen_spatial_demand::{
    ChunkLoadOrder, ChunkStreamingConfig, ChunkStreamingMode, StreamingFocus,
};
use runen_spatial_streaming::{
    ChunkLifecycleState, ProviderEvent, ProviderEventKind, StreamRequest, StreamRequestId,
    StreamRequestKind, StreamingBudgets, StreamingTick, WorldStreamingConfig,
    WorldStreamingController, WorldStreamingError, WorldStreamingEvent, WorldStreamingEventKind,
};

fn partition() -> GridPartitionConfig {
    GridPartitionConfig {
        chunk_edge_meters: 16.0,
        region_chunk_dims: [8, 8, 8],
        fixed_point_scale: 1024,
    }
}

fn chunking_config(load_radius_chunks: i32, unload_radius_chunks: i32) -> ChunkStreamingConfig {
    ChunkStreamingConfig {
        load_radius_chunks,
        unload_radius_chunks,
        vertical_load_radius_chunks: 0,
        vertical_unload_radius_chunks: 0,
        mode: ChunkStreamingMode::PlanarXZ,
        load_order: ChunkLoadOrder::NearestFirst,
    }
}

fn controller(load_budget: usize, unload_budget: usize) -> WorldStreamingController {
    let mut config = WorldStreamingConfig::new(WorldId(7), partition(), chunking_config(1, 1));
    config.budgets = StreamingBudgets {
        max_load_requests_per_tick: load_budget,
        max_unload_requests_per_tick: unload_budget,
    };
    WorldStreamingController::new(config)
}

fn focus(x: f32, y: f32, z: f32) -> StreamingTick {
    StreamingTick::from_focus(StreamingFocus::new([x, y, z]))
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
    let mut config = WorldStreamingConfig::new(WorldId(7), partition(), chunking_config(0, 0));
    config.budgets = StreamingBudgets {
        max_load_requests_per_tick: 1,
        max_unload_requests_per_tick: 1,
    };
    WorldStreamingController::new(config)
}

#[test]
fn tick_emits_budgeted_load_requests_without_loading_payloads() {
    let mut controller = controller(2, 4);

    let output = controller.tick(focus(0.0, 0.0, 0.0));

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
    let output = controller.tick(focus(0.0, 0.0, 0.0));
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
    let mut config = WorldStreamingConfig::new(WorldId(7), partition(), chunking_config(0, 0));
    config.budgets = StreamingBudgets {
        max_load_requests_per_tick: 1,
        max_unload_requests_per_tick: 1,
    };
    let mut controller = WorldStreamingController::new(config);

    let load = controller.tick(focus(0.0, 0.0, 0.0)).requests[0];
    controller
        .accept_provider_event(provider_event(&load, ProviderEventKind::Completed))
        .unwrap();

    let unload_tick = controller.tick(focus(16.0, 0.0, 0.0));
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
    let first = controller.tick(focus(0.0, 0.0, 0.0)).requests[0];

    let events = controller
        .accept_provider_event(provider_event(&first, ProviderEventKind::Failed))
        .unwrap();
    assert_eq!(event_request_ids(&events), vec![Some(first.request_id)]);
    assert_eq!(
        controller.record(first.chunk_id).map(|record| record.state),
        Some(ChunkLifecycleState::Failed)
    );

    let retry = controller.tick(focus(0.0, 0.0, 0.0));
    assert!(retry.requests.is_empty());
}

#[test]
fn explicit_retry_failed_chunk_queues_load_when_still_desired() {
    let mut controller = controller(1, 4);
    let first = controller.tick(focus(0.0, 0.0, 0.0)).requests[0];

    controller
        .accept_provider_event(provider_event(&first, ProviderEventKind::Failed))
        .unwrap();

    let event = controller.retry_failed_chunk(first.chunk_id).unwrap();
    assert_eq!(event.kind, WorldStreamingEventKind::LoadQueued);
    assert_eq!(event.request_id, None);

    let retry = controller.tick(focus(0.0, 0.0, 0.0));
    assert_eq!(retry.requests.len(), 1);
    assert_eq!(retry.requests[0].chunk_id, first.chunk_id);
    assert_eq!(retry.requests[0].kind, StreamRequestKind::Load);
    assert_ne!(retry.requests[0].request_id, first.request_id);
}

#[test]
fn resident_chunk_can_fail_without_payload_ownership() {
    let mut controller = controller(1, 4);
    let request = controller.tick(focus(0.0, 0.0, 0.0)).requests[0];
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
    let load = controller.tick(focus(0.0, 0.0, 0.0)).requests[0];

    controller.tick(focus(16.0, 0.0, 0.0));
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
    let load = controller.tick(focus(0.0, 0.0, 0.0)).requests[0];
    controller
        .accept_provider_event(provider_event(&load, ProviderEventKind::Started))
        .unwrap();

    controller.tick(focus(16.0, 0.0, 0.0));
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
    let load = controller.tick(focus(0.0, 0.0, 0.0)).requests[0];
    controller
        .accept_provider_event(provider_event(&load, ProviderEventKind::Completed))
        .unwrap();
    controller.set_budgets(StreamingBudgets {
        max_load_requests_per_tick: 0,
        max_unload_requests_per_tick: 1,
    });

    let unload = controller
        .tick(focus(16.0, 0.0, 0.0))
        .requests
        .into_iter()
        .find(|request| request.chunk_id == load.chunk_id)
        .unwrap();
    controller.tick(focus(0.0, 0.0, 0.0));
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
    let load = controller.tick(focus(0.0, 0.0, 0.0)).requests[0];
    controller
        .accept_provider_event(provider_event(&load, ProviderEventKind::Completed))
        .unwrap();
    controller.set_budgets(StreamingBudgets {
        max_load_requests_per_tick: 0,
        max_unload_requests_per_tick: 1,
    });

    let unload = controller
        .tick(focus(16.0, 0.0, 0.0))
        .requests
        .into_iter()
        .find(|request| request.chunk_id == load.chunk_id)
        .unwrap();
    controller
        .accept_provider_event(provider_event(&unload, ProviderEventKind::Started))
        .unwrap();
    controller.tick(focus(0.0, 0.0, 0.0));
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
        let load = controller.tick(focus(0.0, 0.0, 0.0)).requests[0];
        controller.tick(focus(16.0, 0.0, 0.0));
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
        .requests
        .into_iter()
        .map(|request| request.chunk_id)
        .collect::<Vec<_>>();
    let second_ids = second
        .tick(focus(0.0, 0.0, 0.0))
        .requests
        .into_iter()
        .map(|request| request.chunk_id)
        .collect::<Vec<_>>();

    assert_eq!(first_ids, second_ids);
}

#[test]
fn stale_provider_event_after_completion_is_rejected() {
    let mut controller = controller(1, 4);
    let request = controller.tick(focus(0.0, 0.0, 0.0)).requests[0];
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
