use runen_spatial::{GridPartitionConfig, WorldId, WorldPosition};
use runen_spatial_demand::{
    DemandDistanceOrder, DemandFocus, DemandLimits, DemandSourceChange, DemandSourceId,
    DemandSourcePriority, DemandSourceSnapshot, DemandTransaction,
};
use runen_spatial_streaming::{
    ProviderEvent, ProviderEventKind, StreamRequest, StreamRequestKind, StreamingBudgets,
    StreamingTick, WorldStreamingConfig, WorldStreamingController,
};

fn main() {
    let mut controller = WorldStreamingController::new(streaming_config());

    let position =
        WorldPosition::try_new(WorldId(0), [0.0, 0.0, 0.0]).expect("demo position is valid");
    let tick = controller
        .tick(StreamingTick::from_demand_transaction(demo_transaction(
            position,
        )))
        .expect("demo tick is valid");

    println!("Stream requests:");
    for request in &tick.requests {
        print_stream_request(request);
    }

    let Some(load_request) = tick
        .requests
        .iter()
        .find(|request| request.kind == StreamRequestKind::Load)
        .copied()
    else {
        println!("No load request emitted.");
        return;
    };

    controller
        .accept_provider_event(ProviderEvent {
            request_id: load_request.request_id,
            chunk_id: load_request.chunk_id,
            kind: ProviderEventKind::Started,
        })
        .expect("provider start should match pending request");
    let events = controller
        .accept_provider_event(ProviderEvent {
            request_id: load_request.request_id,
            chunk_id: load_request.chunk_id,
            kind: ProviderEventKind::Completed,
        })
        .expect("provider completion should match pending request");

    println!();
    println!("Lifecycle events after provider completion:");
    for event in events {
        println!("  {:?} {:?}", event.kind, event.chunk_id.coord);
    }
}

fn streaming_config() -> WorldStreamingConfig {
    let mut config = WorldStreamingConfig::new(
        WorldId(0),
        GridPartitionConfig::try_new(32.0, [8, 8, 8]).expect("demo partition is valid"),
        DemandLimits::default(),
    );
    config.budgets = StreamingBudgets {
        max_load_requests_per_tick: 1,
        max_unload_requests_per_tick: 1,
    };
    config
}

fn demo_transaction(position: WorldPosition) -> DemandTransaction {
    let focus = DemandFocus::try_new(position, 0, 1, 0, 0, DemandDistanceOrder::NearestFirst)
        .expect("demo focus is valid");
    let snapshot = DemandSourceSnapshot::try_new(DemandSourcePriority::new(0), Some(focus), [])
        .expect("demo snapshot is valid");
    DemandTransaction::try_new([DemandSourceChange::Replace {
        source_id: DemandSourceId::new(0),
        snapshot,
    }])
    .expect("demo transaction is valid")
}

fn print_stream_request(request: &StreamRequest) {
    let coord = request.chunk_id.coord;
    println!(
        "  id={} kind={:?} chunk=({}, {}, {}) priority={:?}",
        request.request_id.0, request.kind, coord.x, coord.y, coord.z, request.priority
    );
}
