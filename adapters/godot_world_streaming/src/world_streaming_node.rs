use godot::builtin::{Dictionary, GString, Variant, Vector3};
use godot::classes::{INode, Node};
use godot::prelude::*;
use runen_spatial::{ChunkId, GridPartitionConfig, WorldId};
use runen_spatial_demand::{
    ChunkLoadOrder, ChunkStreamingConfig, ChunkStreamingMode, StreamingFocus,
};
use runen_spatial_streaming::{
    ChunkLifecycleState, ProviderEvent, ProviderEventKind, StreamRequest, StreamRequestId,
    StreamRequestKind, StreamingBudgets, StreamingTick, WorldStreamingConfig,
    WorldStreamingController, WorldStreamingEvent, WorldStreamingEventKind,
};

use crate::bridge::{provider_event_from_godot, vector3_to_meters};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GodotWorldStreamingNode {
    base: Base<Node>,

    world_id: u16,
    chunk_edge_meters: f32,
    region_dim_x: u32,
    region_dim_y: u32,
    region_dim_z: u32,
    fixed_point_scale: i32,

    load_radius_chunks: i32,
    unload_radius_chunks: i32,
    vertical_load_radius_chunks: i32,
    vertical_unload_radius_chunks: i32,
    planar_xz_mode: bool,

    max_load_requests_per_tick: usize,
    max_unload_requests_per_tick: usize,

    controller: Option<WorldStreamingController>,
}

#[godot_api]
impl INode for GodotWorldStreamingNode {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            world_id: 0,
            chunk_edge_meters: 32.0,
            region_dim_x: 8,
            region_dim_y: 8,
            region_dim_z: 8,
            fixed_point_scale: 1024,
            load_radius_chunks: 4,
            unload_radius_chunks: 6,
            vertical_load_radius_chunks: 1,
            vertical_unload_radius_chunks: 2,
            planar_xz_mode: true,
            max_load_requests_per_tick: 4,
            max_unload_requests_per_tick: 4,
            controller: None,
        }
    }

    fn ready(&mut self) {
        self.rebuild_controller();
    }
}

#[godot_api]
impl GodotWorldStreamingNode {
    #[signal]
    fn chunk_load_requested(request_id: i64, x: i32, y: i32, z: i32);

    #[signal]
    fn chunk_provider_started(request_id: i64, x: i32, y: i32, z: i32);

    #[signal]
    fn chunk_provider_completed(request_id: i64, x: i32, y: i32, z: i32);

    #[signal]
    fn chunk_provider_failed(request_id: i64, x: i32, y: i32, z: i32);

    #[signal]
    fn chunk_resident(x: i32, y: i32, z: i32);

    #[signal]
    fn chunk_unload_requested(request_id: i64, x: i32, y: i32, z: i32);

    #[signal]
    fn chunk_unloaded(x: i32, y: i32, z: i32);

    #[signal]
    fn streaming_error(message: GString);

    #[func]
    pub fn set_world_id(&mut self, value: i32) {
        self.world_id = value.clamp(0, i32::from(u16::MAX)) as u16;
        self.rebuild_controller();
    }

    #[func]
    pub fn get_world_id(&self) -> i32 {
        i32::from(self.world_id)
    }

    #[func]
    pub fn set_chunk_edge_meters(&mut self, value: f32) {
        self.chunk_edge_meters = value.max(1.0);
        self.rebuild_controller();
    }

    #[func]
    pub fn get_chunk_edge_meters(&self) -> f32 {
        self.chunk_edge_meters
    }

    #[func]
    pub fn set_region_chunk_dims(&mut self, x: i32, y: i32, z: i32) {
        self.region_dim_x = x.max(1) as u32;
        self.region_dim_y = y.max(1) as u32;
        self.region_dim_z = z.max(1) as u32;
        self.rebuild_controller();
    }

    #[func]
    pub fn set_fixed_point_scale(&mut self, value: i32) {
        self.fixed_point_scale = value.max(1);
        self.rebuild_controller();
    }

    #[func]
    pub fn set_load_radii(
        &mut self,
        load_radius_chunks: i32,
        unload_radius_chunks: i32,
        vertical_load_radius_chunks: i32,
        vertical_unload_radius_chunks: i32,
    ) {
        self.load_radius_chunks = load_radius_chunks;
        self.unload_radius_chunks = unload_radius_chunks;
        self.vertical_load_radius_chunks = vertical_load_radius_chunks;
        self.vertical_unload_radius_chunks = vertical_unload_radius_chunks;
        self.rebuild_controller();
    }

    #[func]
    pub fn set_request_budgets(
        &mut self,
        max_load_requests_per_tick: i32,
        max_unload_requests_per_tick: i32,
    ) {
        self.max_load_requests_per_tick = max_load_requests_per_tick.max(0) as usize;
        self.max_unload_requests_per_tick = max_unload_requests_per_tick.max(0) as usize;
        let budgets = self.streaming_budgets();
        if let Some(controller) = &mut self.controller {
            controller.set_budgets(budgets);
        }
    }

    #[func]
    pub fn set_planar_xz_mode(&mut self) {
        self.planar_xz_mode = true;
        self.rebuild_controller();
    }

    #[func]
    pub fn set_volume_3d_mode(&mut self) {
        self.planar_xz_mode = false;
        self.rebuild_controller();
    }

    #[func]
    pub fn reset_streaming_state(&mut self) {
        self.rebuild_controller();
    }

    #[func]
    pub fn update_focus_from_vector3(&mut self, position: Vector3) {
        let Some(controller) = &mut self.controller else {
            return;
        };

        let output = controller.tick(StreamingTick::from_focus(StreamingFocus::new(
            vector3_to_meters(position),
        )));
        self.emit_tick_output(output.requests, output.events);
    }

    #[func]
    pub fn provider_started(&mut self, request_id: i64, x: i32, y: i32, z: i32) {
        self.accept_provider_event_from_godot(request_id, x, y, z, ProviderEventKind::Started);
    }

    #[func]
    pub fn provider_completed(&mut self, request_id: i64, x: i32, y: i32, z: i32) {
        self.accept_provider_event_from_godot(request_id, x, y, z, ProviderEventKind::Completed);
    }

    #[func]
    pub fn provider_failed(&mut self, request_id: i64, x: i32, y: i32, z: i32) {
        self.accept_provider_event_from_godot(request_id, x, y, z, ProviderEventKind::Failed);
    }

    #[func]
    pub fn resident_chunk_count(&self) -> i64 {
        self.count_records_with_state(ChunkLifecycleState::Resident)
    }

    #[func]
    pub fn pending_request_count(&self) -> i64 {
        self.controller
            .as_ref()
            .map(|controller| controller.pending_requests().count() as i64)
            .unwrap_or(0)
    }

    #[func]
    pub fn tracked_chunk_count(&self) -> i64 {
        self.controller
            .as_ref()
            .map(|controller| controller.records().count() as i64)
            .unwrap_or(0)
    }

    #[func]
    pub fn describe_config(&self) -> Dictionary<Variant, Variant> {
        let mut dict = Dictionary::<Variant, Variant>::new();
        dict.set("world_id", i64::from(self.world_id));
        dict.set("chunk_edge_meters", self.chunk_edge_meters);
        dict.set("region_dim_x", self.region_dim_x as i64);
        dict.set("region_dim_y", self.region_dim_y as i64);
        dict.set("region_dim_z", self.region_dim_z as i64);
        dict.set("fixed_point_scale", self.fixed_point_scale);
        dict.set("load_radius_chunks", self.load_radius_chunks);
        dict.set("unload_radius_chunks", self.unload_radius_chunks);
        dict.set(
            "vertical_load_radius_chunks",
            self.vertical_load_radius_chunks,
        );
        dict.set(
            "vertical_unload_radius_chunks",
            self.vertical_unload_radius_chunks,
        );
        dict.set(
            "streaming_mode",
            if self.planar_xz_mode {
                "planar_xz"
            } else {
                "volume_3d"
            },
        );
        dict.set(
            "max_load_requests_per_tick",
            self.max_load_requests_per_tick as i64,
        );
        dict.set(
            "max_unload_requests_per_tick",
            self.max_unload_requests_per_tick as i64,
        );
        dict
    }

    fn accept_provider_event_from_godot(
        &mut self,
        request_id: i64,
        x: i32,
        y: i32,
        z: i32,
        kind: ProviderEventKind,
    ) {
        let Some(event) = provider_event_from_godot(self.world_id, request_id, x, y, z, kind)
        else {
            let message = GString::from("request_id must be non-negative");
            self.signals().streaming_error().emit(&message);
            return;
        };
        self.accept_provider_event(event);
    }

    fn accept_provider_event(&mut self, event: ProviderEvent) {
        let Some(controller) = &mut self.controller else {
            return;
        };

        match controller.accept_provider_event(event) {
            Ok(events) => {
                for world_event in events {
                    self.emit_world_event(world_event);
                }
            }
            Err(error) => {
                let message = format!("{error:?}");
                let message = GString::from(message.as_str());
                self.signals().streaming_error().emit(&message);
            }
        }
    }

    fn emit_tick_output(&mut self, requests: Vec<StreamRequest>, events: Vec<WorldStreamingEvent>) {
        for request in requests {
            self.emit_stream_request(request);
        }

        for event in events {
            self.emit_world_event(event);
        }
    }

    fn emit_stream_request(&mut self, request: StreamRequest) {
        let request_id = request_id_to_i64(request.request_id);
        let chunk = request.chunk_id.coord;
        match request.kind {
            StreamRequestKind::Load => self
                .signals()
                .chunk_load_requested()
                .emit(request_id, chunk.x, chunk.y, chunk.z),
            StreamRequestKind::Unload => self
                .signals()
                .chunk_unload_requested()
                .emit(request_id, chunk.x, chunk.y, chunk.z),
        }
    }

    fn emit_world_event(&mut self, event: WorldStreamingEvent) {
        let chunk = event.chunk_id.coord;
        match event.kind {
            WorldStreamingEventKind::ProviderStarted => {
                if let Some(request_id) = event.request_id {
                    self.signals().chunk_provider_started().emit(
                        request_id_to_i64(request_id),
                        chunk.x,
                        chunk.y,
                        chunk.z,
                    );
                }
            }
            WorldStreamingEventKind::ProviderCompleted => {
                if let Some(request_id) = event.request_id {
                    self.signals().chunk_provider_completed().emit(
                        request_id_to_i64(request_id),
                        chunk.x,
                        chunk.y,
                        chunk.z,
                    );
                }
            }
            WorldStreamingEventKind::ProviderFailed => {
                if let Some(request_id) = event.request_id {
                    self.signals().chunk_provider_failed().emit(
                        request_id_to_i64(request_id),
                        chunk.x,
                        chunk.y,
                        chunk.z,
                    );
                }
            }
            WorldStreamingEventKind::Resident => {
                self.signals()
                    .chunk_resident()
                    .emit(chunk.x, chunk.y, chunk.z);
            }
            WorldStreamingEventKind::Unloaded => {
                self.signals()
                    .chunk_unloaded()
                    .emit(chunk.x, chunk.y, chunk.z);
            }
            WorldStreamingEventKind::LoadQueued
            | WorldStreamingEventKind::LoadRequested
            | WorldStreamingEventKind::UnloadQueued
            | WorldStreamingEventKind::UnloadRequested => {}
        }
    }

    fn rebuild_controller(&mut self) {
        self.controller = Some(WorldStreamingController::new(self.streaming_config()));
    }

    fn streaming_config(&self) -> WorldStreamingConfig {
        let partition = GridPartitionConfig {
            chunk_edge_meters: self.chunk_edge_meters.max(1.0),
            region_chunk_dims: [
                self.region_dim_x.max(1),
                self.region_dim_y.max(1),
                self.region_dim_z.max(1),
            ],
            fixed_point_scale: self.fixed_point_scale.max(1),
        };

        let mut config = WorldStreamingConfig::new(
            WorldId(self.world_id),
            partition,
            ChunkStreamingConfig {
                load_radius_chunks: self.load_radius_chunks,
                unload_radius_chunks: self.unload_radius_chunks,
                vertical_load_radius_chunks: self.vertical_load_radius_chunks,
                vertical_unload_radius_chunks: self.vertical_unload_radius_chunks,
                mode: if self.planar_xz_mode {
                    ChunkStreamingMode::PlanarXZ
                } else {
                    ChunkStreamingMode::Volume3D
                },
                load_order: ChunkLoadOrder::NearestFirst,
            },
        );
        config.budgets = self.streaming_budgets();
        config
    }

    fn streaming_budgets(&self) -> StreamingBudgets {
        StreamingBudgets {
            max_load_requests_per_tick: self.max_load_requests_per_tick,
            max_unload_requests_per_tick: self.max_unload_requests_per_tick,
        }
    }

    fn count_records_with_state(&self, state: ChunkLifecycleState) -> i64 {
        self.controller
            .as_ref()
            .map(|controller| {
                controller
                    .records()
                    .filter(|record| record.state == state)
                    .count() as i64
            })
            .unwrap_or(0)
    }
}

fn request_id_to_i64(request_id: StreamRequestId) -> i64 {
    i64::try_from(request_id.0).unwrap_or(i64::MAX)
}

#[allow(dead_code)]
fn chunk_id_to_xyz(chunk_id: ChunkId) -> (i32, i32, i32) {
    (chunk_id.coord.x, chunk_id.coord.y, chunk_id.coord.z)
}
