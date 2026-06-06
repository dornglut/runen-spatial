use chunking::{ChunkLoadOrder, ChunkStreamingConfig, ChunkStreamingMode, StreamingFocus};
use spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
use world_streaming::{
    ChunkLifecycleState, ProviderEvent, ProviderEventKind, StreamRequestKind, StreamingBudgets,
    StreamingTick, WorldStreamingConfig, WorldStreamingController, WorldStreamingError,
    WorldStreamingEventKind,
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

fn provider_event(
    request: &world_streaming::StreamRequest,
    kind: ProviderEventKind,
) -> ProviderEvent {
    ProviderEvent {
        request_id: request.request_id,
        chunk_id: request.chunk_id,
        kind,
    }
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
fn failed_load_retries_when_still_desired() {
    let mut controller = controller(1, 4);
    let first = controller.tick(focus(0.0, 0.0, 0.0)).requests[0];

    controller
        .accept_provider_event(provider_event(&first, ProviderEventKind::Failed))
        .unwrap();
    assert_eq!(
        controller.record(first.chunk_id).map(|record| record.state),
        Some(ChunkLifecycleState::Failed)
    );

    let retry = controller.tick(focus(0.0, 0.0, 0.0));
    assert_eq!(retry.requests.len(), 1);
    assert_eq!(retry.requests[0].chunk_id, first.chunk_id);
    assert_eq!(retry.requests[0].kind, StreamRequestKind::Load);
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
    assert_eq!(
        controller
            .record(request.chunk_id)
            .map(|record| record.state),
        Some(ChunkLifecycleState::Failed)
    );
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
