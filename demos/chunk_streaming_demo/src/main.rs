use world_core_prelude::{
    ChunkLoadOrder, ChunkStreamingConfig, ChunkStreamingMode, GridPartitionConfig, ProviderEvent,
    ProviderEventKind, StreamRequest, StreamRequestKind, StreamingBudgets, StreamingFocus,
    StreamingTick, WorldId, WorldStreamingConfig, WorldStreamingController,
};

fn main() {
    let mut controller = WorldStreamingController::new(streaming_config());

    let tick = controller.tick(StreamingTick::from_focus(StreamingFocus::new([
        0.0, 0.0, 0.0,
    ])));

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
        GridPartitionConfig {
            chunk_edge_meters: 32.0,
            region_chunk_dims: [8, 8, 8],
            fixed_point_scale: 1024,
        },
        ChunkStreamingConfig {
            load_radius_chunks: 0,
            unload_radius_chunks: 1,
            vertical_load_radius_chunks: 0,
            vertical_unload_radius_chunks: 0,
            mode: ChunkStreamingMode::PlanarXZ,
            load_order: ChunkLoadOrder::NearestFirst,
        },
    );
    config.budgets = StreamingBudgets {
        max_load_requests_per_tick: 1,
        max_unload_requests_per_tick: 1,
    };
    config
}

fn print_stream_request(request: &StreamRequest) {
    let coord = request.chunk_id.coord;
    println!(
        "  id={} kind={:?} chunk=({}, {}, {}) priority={:?}",
        request.request_id.0, request.kind, coord.x, coord.y, coord.z, request.priority
    );
}
