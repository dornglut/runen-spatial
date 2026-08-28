use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId, WorldPosition};
use runen_spatial_demand::{
    DemandFocus, DemandLimits, DemandSourceChange, DemandSourceId, DemandSourceSnapshot,
    EffectiveDemandSnapshot,
};
use runen_spatial_streaming::{
    ChunkAvailability, ChunkRuntimeRecord, ProviderEvent, ProviderEventKind, StreamRequest,
    StreamRequestKind, StreamingBudgets, StreamingCapacity, StreamingTick, StreamingTickOutput,
    WorldStreamingConfig, WorldStreamingController, WorldStreamingEvent,
};

#[derive(Debug, PartialEq, Eq)]
struct ReplayObservation {
    tick_outputs: Vec<StreamingTickOutput>,
    provider_events: Vec<Vec<WorldStreamingEvent>>,
    effective_demand: EffectiveDemandSnapshot,
    records: Vec<ChunkRuntimeRecord>,
    pending_requests: Vec<StreamRequest>,
}

fn chunk(x: i64) -> ChunkId {
    ChunkId::new(WorldId::new(7), ChunkCoord3 { x, y: 0, z: 0 })
}

fn source_focus(source_id: u64, x: f64) -> DemandSourceChange {
    let focus = DemandFocus::try_new(
        WorldPosition::try_new(WorldId::new(7), [x, 0.0, 0.0]).unwrap(),
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

fn provider_event(request: StreamRequest, kind: ProviderEventKind) -> ProviderEvent {
    ProviderEvent {
        request_id: request.request_id,
        chunk_id: request.chunk_id,
        kind,
    }
}

fn run_replay() -> ReplayObservation {
    let mut config = WorldStreamingConfig::new(
        WorldId::new(7),
        GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap(),
        DemandLimits::default(),
        StreamingCapacity::new(2, 1, 1),
    );
    config.budgets = StreamingBudgets {
        max_load_requests_per_tick: 2,
        max_unload_requests_per_tick: 1,
    };
    let mut controller = WorldStreamingController::new(config);
    let mut tick_outputs = Vec::new();
    let mut provider_events = Vec::new();

    let initial = controller
        .tick(StreamingTick::from_demand_changes([
            source_focus(1, 0.0),
            source_focus(2, 16.0),
        ]))
        .unwrap();
    assert_eq!(initial.requests.len(), 1);
    assert_eq!(initial.requests[0].kind, StreamRequestKind::Load);
    assert_eq!(initial.requests[0].chunk_id, chunk(0));
    assert_eq!(initial.pressure.deferred_loads(), 1);
    let first_load = initial.requests[0];
    tick_outputs.push(initial);

    provider_events.push(
        controller
            .accept_provider_event(provider_event(first_load, ProviderEventKind::Started))
            .unwrap(),
    );
    provider_events.push(
        controller
            .accept_provider_event(provider_event(first_load, ProviderEventKind::Completed))
            .unwrap(),
    );

    let second = controller
        .tick(StreamingTick::without_demand_changes())
        .unwrap();
    assert_eq!(second.requests.len(), 1);
    assert_eq!(second.requests[0].kind, StreamRequestKind::Load);
    assert_eq!(second.requests[0].chunk_id, chunk(1));
    let second_load = second.requests[0];
    tick_outputs.push(second);

    provider_events.push(
        controller
            .accept_provider_event(provider_event(second_load, ProviderEventKind::Started))
            .unwrap(),
    );

    let removal = controller
        .tick(StreamingTick::from_demand_changes([
            DemandSourceChange::Remove {
                source_id: DemandSourceId::new(1),
            },
        ]))
        .unwrap();
    assert_eq!(removal.requests.len(), 1);
    assert_eq!(removal.requests[0].kind, StreamRequestKind::Unload);
    assert_eq!(removal.requests[0].chunk_id, chunk(0));
    assert_eq!(removal.pressure.in_flight_loads(), 1);
    assert_eq!(removal.pressure.in_flight_unloads(), 1);
    let first_unload = removal.requests[0];
    tick_outputs.push(removal);

    provider_events.push(
        controller
            .accept_provider_event(provider_event(second_load, ProviderEventKind::Completed))
            .unwrap(),
    );
    provider_events.push(
        controller
            .accept_provider_event(provider_event(first_unload, ProviderEventKind::Started))
            .unwrap(),
    );
    provider_events.push(
        controller
            .accept_provider_event(provider_event(first_unload, ProviderEventKind::Completed))
            .unwrap(),
    );

    let settled = controller
        .tick(StreamingTick::without_demand_changes())
        .unwrap();
    assert!(settled.requests.is_empty());
    tick_outputs.push(settled);

    let effective_demand = controller.effective_demand().clone();
    let records = controller.records().copied().collect::<Vec<_>>();
    let pending_requests = controller.pending_requests().copied().collect::<Vec<_>>();

    assert_eq!(effective_demand.chunks().len(), 1);
    assert_eq!(effective_demand.chunks()[0].chunk_id(), chunk(1));
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].chunk_id(), chunk(1));
    assert_eq!(records[0].availability(), ChunkAvailability::Resident);
    assert!(pending_requests.is_empty());

    ReplayObservation {
        tick_outputs,
        provider_events,
        effective_demand,
        records,
        pending_requests,
    }
}

#[test]
fn public_api_replay_is_deterministic_across_fresh_controllers() {
    assert_eq!(run_replay(), run_replay());
}
