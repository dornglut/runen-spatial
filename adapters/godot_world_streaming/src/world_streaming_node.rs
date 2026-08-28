use godot::builtin::{Dictionary, GString, Variant, Vector3};
use godot::classes::{INode, Node};
use godot::prelude::*;
use runen_spatial::{ChunkId, GridPartitionConfig, SpatialMathError, WorldId, WorldPosition};
use runen_spatial_demand::{
    DemandFocus, DemandLimits, DemandSourceChange, DemandSourceId, DemandSourceSnapshot,
    SpatialDemandError,
};
use runen_spatial_streaming::{
    ChunkAvailability, ProviderEvent, ProviderEventKind, StreamRequest, StreamRequestId,
    StreamRequestKind, StreamingBudgets, StreamingCapacity, StreamingTick, WorldStreamingConfig,
    WorldStreamingController, WorldStreamingEvent, WorldStreamingEventKind,
};

use crate::bridge::{provider_event_from_godot, vector3_to_meters};

const ADAPTER_STREAMING_CAPACITY: StreamingCapacity = StreamingCapacity::new(1024, 4, 4);
const NODE_FOCUS_SOURCE: DemandSourceId = DemandSourceId::new(0);

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GodotWorldStreamingNode {
    base: Base<Node>,

    world_id: u16,
    chunk_edge_meters: f32,
    region_dim_x: u32,
    region_dim_y: u32,
    region_dim_z: u32,

    load_radius_chunks: i32,
    unload_radius_chunks: i32,
    vertical_load_radius_chunks: i32,
    vertical_unload_radius_chunks: i32,

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
            load_radius_chunks: 4,
            unload_radius_chunks: 6,
            vertical_load_radius_chunks: 1,
            vertical_unload_radius_chunks: 2,
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
    fn chunk_load_requested(request_id: i64, x: i64, y: i64, z: i64);

    #[signal]
    fn chunk_provider_started(request_id: i64, x: i64, y: i64, z: i64);

    #[signal]
    fn chunk_provider_completed(request_id: i64, x: i64, y: i64, z: i64);

    #[signal]
    fn chunk_provider_failed(request_id: i64, x: i64, y: i64, z: i64);

    #[signal]
    fn chunk_resident(x: i64, y: i64, z: i64);

    #[signal]
    fn chunk_unload_requested(request_id: i64, x: i64, y: i64, z: i64);

    #[signal]
    fn chunk_unloaded(x: i64, y: i64, z: i64);

    #[signal]
    fn streaming_error(message: GString);

    #[func]
    pub fn set_world_id(&mut self, value: i32) {
        let world_id = match requested_world_id(value) {
            Ok(world_id) => world_id,
            Err(error) => {
                self.report_spatial_error(error);
                return;
            }
        };
        let config = match streaming_config_from_values(
            world_id,
            self.chunk_edge_meters,
            [self.region_dim_x, self.region_dim_y, self.region_dim_z],
            self.requested_radii_values(),
            self.streaming_budgets(),
        ) {
            Ok(config) => config,
            Err(error) => {
                self.report_demand_error(error);
                return;
            }
        };
        if !self.controller_rebuild_allowed("set_world_id") {
            return;
        }
        self.world_id = world_id;
        self.controller = Some(WorldStreamingController::new(config));
    }

    #[func]
    pub fn get_world_id(&self) -> i32 {
        i32::from(self.world_id)
    }

    #[func]
    pub fn set_chunk_edge_meters(&mut self, value: f32) {
        let config = match streaming_config_from_values(
            self.world_id,
            value,
            [self.region_dim_x, self.region_dim_y, self.region_dim_z],
            self.requested_radii_values(),
            self.streaming_budgets(),
        ) {
            Ok(config) => config,
            Err(error) => {
                self.report_demand_error(error);
                return;
            }
        };
        if !self.controller_rebuild_allowed("set_chunk_edge_meters") {
            return;
        }
        self.chunk_edge_meters = value;
        self.controller = Some(WorldStreamingController::new(config));
    }

    #[func]
    pub fn get_chunk_edge_meters(&self) -> f32 {
        self.chunk_edge_meters
    }

    #[func]
    pub fn set_region_chunk_dims(&mut self, x: i32, y: i32, z: i32) {
        let dims = match requested_region_dims([x, y, z]) {
            Ok(dims) => dims,
            Err(error) => {
                self.report_spatial_error(error);
                return;
            }
        };
        let config = match streaming_config_from_values(
            self.world_id,
            self.chunk_edge_meters,
            dims,
            self.requested_radii_values(),
            self.streaming_budgets(),
        ) {
            Ok(config) => config,
            Err(error) => {
                self.report_demand_error(error);
                return;
            }
        };
        if !self.controller_rebuild_allowed("set_region_chunk_dims") {
            return;
        }
        [self.region_dim_x, self.region_dim_y, self.region_dim_z] = dims;
        self.controller = Some(WorldStreamingController::new(config));
    }

    #[func]
    pub fn set_load_radii(
        &mut self,
        load_radius_chunks: i32,
        unload_radius_chunks: i32,
        vertical_load_radius_chunks: i32,
        vertical_unload_radius_chunks: i32,
    ) {
        let requested = [
            load_radius_chunks,
            unload_radius_chunks,
            vertical_load_radius_chunks,
            vertical_unload_radius_chunks,
        ];
        if let Err(error) = requested_radii(requested) {
            self.report_demand_error(error);
            return;
        }
        self.load_radius_chunks = load_radius_chunks;
        self.unload_radius_chunks = unload_radius_chunks;
        self.vertical_load_radius_chunks = vertical_load_radius_chunks;
        self.vertical_unload_radius_chunks = vertical_unload_radius_chunks;
    }

    #[func]
    pub fn set_request_budgets(
        &mut self,
        max_load_requests_per_tick: i32,
        max_unload_requests_per_tick: i32,
    ) {
        let budgets =
            match requested_budgets([max_load_requests_per_tick, max_unload_requests_per_tick]) {
                Ok(budgets) => budgets,
                Err(message) => {
                    self.report_adapter_error(message);
                    return;
                }
            };
        self.max_load_requests_per_tick = budgets.max_load_requests_per_tick;
        self.max_unload_requests_per_tick = budgets.max_unload_requests_per_tick;
        if let Some(controller) = &mut self.controller {
            controller.set_budgets(budgets);
        }
    }

    #[func]
    pub fn reset_streaming_state(&mut self) {
        if !self.controller_rebuild_allowed("reset_streaming_state") {
            return;
        }
        self.rebuild_controller();
    }

    #[func]
    pub fn update_focus_from_vector3(&mut self, position: Vector3) {
        let Some(controller) = self.controller.as_ref() else {
            return;
        };

        let position =
            match WorldPosition::try_new(controller.world_id(), vector3_to_meters(position)) {
                Ok(position) => position,
                Err(error) => {
                    let message = GString::from(format!("{error:?}").as_str());
                    self.signals().streaming_error().emit(&message);
                    return;
                }
            };
        let changes = match demand_changes_from_node_values(position, self.requested_radii_values())
        {
            Ok(transaction) => transaction,
            Err(error) => {
                self.report_demand_error(error);
                return;
            }
        };
        let Some(controller) = &mut self.controller else {
            return;
        };
        match controller.tick(StreamingTick::from_demand_changes(changes)) {
            Ok(output) => self.emit_tick_output(output.requests, output.request_id_exhausted),
            Err(error) => {
                let message = GString::from(format!("{error:?}").as_str());
                self.signals().streaming_error().emit(&message);
            }
        }
    }

    #[func]
    pub fn provider_started(&mut self, request_id: i64, x: i64, y: i64, z: i64) {
        self.accept_provider_event_from_godot(request_id, x, y, z, ProviderEventKind::Started);
    }

    #[func]
    pub fn provider_completed(&mut self, request_id: i64, x: i64, y: i64, z: i64) {
        self.accept_provider_event_from_godot(request_id, x, y, z, ProviderEventKind::Completed);
    }

    #[func]
    pub fn provider_failed(&mut self, request_id: i64, x: i64, y: i64, z: i64) {
        self.accept_provider_event_from_godot(request_id, x, y, z, ProviderEventKind::Failed);
    }

    #[func]
    pub fn resident_chunk_count(&self) -> i64 {
        self.count_records_with_availability(ChunkAvailability::Resident)
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
        x: i64,
        y: i64,
        z: i64,
        kind: ProviderEventKind,
    ) {
        let Some(event) = provider_event_from_godot(self.world_id, request_id, x, y, z, kind)
        else {
            let message = GString::from("request_id must be a positive signed 64-bit integer");
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

    fn emit_tick_output(&mut self, requests: Vec<StreamRequest>, request_id_exhausted: bool) {
        for request in requests {
            self.emit_stream_request(request);
        }

        if request_id_exhausted {
            let message = GString::from("stream request ID space exhausted");
            self.signals().streaming_error().emit(&message);
        }
    }

    fn emit_stream_request(&mut self, request: StreamRequest) {
        let Some(request_id) = request_id_to_i64(request.request_id) else {
            self.report_request_id_error(request.request_id);
            return;
        };
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
                let Some(request_id) = request_id_to_i64(event.request_id) else {
                    self.report_request_id_error(event.request_id);
                    return;
                };
                self.signals()
                    .chunk_provider_started()
                    .emit(request_id, chunk.x, chunk.y, chunk.z);
            }
            WorldStreamingEventKind::ProviderCompleted => {
                let Some(request_id) = request_id_to_i64(event.request_id) else {
                    self.report_request_id_error(event.request_id);
                    return;
                };
                self.signals()
                    .chunk_provider_completed()
                    .emit(request_id, chunk.x, chunk.y, chunk.z);
            }
            WorldStreamingEventKind::ProviderFailed => {
                let Some(request_id) = request_id_to_i64(event.request_id) else {
                    self.report_request_id_error(event.request_id);
                    return;
                };
                self.signals()
                    .chunk_provider_failed()
                    .emit(request_id, chunk.x, chunk.y, chunk.z);
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
        }
    }

    fn rebuild_controller(&mut self) {
        match self.streaming_config() {
            Ok(config) => self.controller = Some(WorldStreamingController::new(config)),
            Err(error) => self.report_demand_error(error),
        }
    }

    fn streaming_config(&self) -> Result<WorldStreamingConfig, SpatialDemandError> {
        streaming_config_from_values(
            self.world_id,
            self.chunk_edge_meters,
            [self.region_dim_x, self.region_dim_y, self.region_dim_z],
            self.requested_radii_values(),
            self.streaming_budgets(),
        )
    }

    fn requested_radii_values(&self) -> [i32; 4] {
        [
            self.load_radius_chunks,
            self.unload_radius_chunks,
            self.vertical_load_radius_chunks,
            self.vertical_unload_radius_chunks,
        ]
    }

    fn controller_rebuild_allowed(&mut self, operation: &str) -> bool {
        if controller_has_runtime_records(self.controller.as_ref()) {
            let message = format!(
                "{operation} requires empty streaming runtime state; complete or unload tracked chunks before rebuilding"
            );
            let message = GString::from(message.as_str());
            self.signals().streaming_error().emit(&message);
            return false;
        }
        true
    }

    fn report_adapter_error(&mut self, message: &str) {
        let message = GString::from(message);
        self.signals().streaming_error().emit(&message);
    }

    fn report_spatial_error(&mut self, error: SpatialMathError) {
        let message = GString::from(format!("{error:?}").as_str());
        self.signals().streaming_error().emit(&message);
    }

    fn report_demand_error(&mut self, error: SpatialDemandError) {
        let message = GString::from(format!("{error:?}").as_str());
        self.signals().streaming_error().emit(&message);
    }

    fn report_request_id_error(&mut self, request_id: StreamRequestId) {
        let message = format!(
            "request_id {} cannot be represented exactly as a Godot i64",
            request_id.get()
        );
        let message = GString::from(message.as_str());
        self.signals().streaming_error().emit(&message);
    }

    fn streaming_budgets(&self) -> StreamingBudgets {
        StreamingBudgets {
            max_load_requests_per_tick: self.max_load_requests_per_tick,
            max_unload_requests_per_tick: self.max_unload_requests_per_tick,
        }
    }

    fn count_records_with_availability(&self, availability: ChunkAvailability) -> i64 {
        self.controller
            .as_ref()
            .map(|controller| {
                controller
                    .records()
                    .filter(|record| record.availability() == availability)
                    .count() as i64
            })
            .unwrap_or(0)
    }
}

fn requested_world_id(value: i32) -> Result<u16, SpatialMathError> {
    u16::try_from(value).map_err(|_| SpatialMathError::CoordinateOutOfRange {
        operation: "Godot world_id",
    })
}

fn requested_region_dims(requested: [i32; 3]) -> Result<[u32; 3], SpatialMathError> {
    for (axis, value) in requested.iter().enumerate() {
        if *value <= 0 {
            return Err(SpatialMathError::NonPositiveValue {
                field: match axis {
                    0 => "region_dim_x",
                    1 => "region_dim_y",
                    _ => "region_dim_z",
                },
            });
        }
    }
    Ok([
        u32::try_from(requested[0]).map_err(|_| SpatialMathError::CoordinateOutOfRange {
            operation: "region_dim_x",
        })?,
        u32::try_from(requested[1]).map_err(|_| SpatialMathError::CoordinateOutOfRange {
            operation: "region_dim_y",
        })?,
        u32::try_from(requested[2]).map_err(|_| SpatialMathError::CoordinateOutOfRange {
            operation: "region_dim_z",
        })?,
    ])
}

fn partition_from_node_values(
    chunk_edge_meters: f32,
    region_chunk_dims: [u32; 3],
) -> Result<GridPartitionConfig, SpatialMathError> {
    GridPartitionConfig::try_new(f64::from(chunk_edge_meters), region_chunk_dims)
}

fn requested_radii(requested: [i32; 4]) -> Result<[u32; 4], SpatialDemandError> {
    let [
        horizontal_desired,
        horizontal_retain,
        vertical_desired,
        vertical_retain,
    ] = requested;
    let radii = [
        u32::try_from(horizontal_desired).map_err(|_| SpatialDemandError::CountOverflow {
            operation: "Godot demand radius",
        })?,
        u32::try_from(horizontal_retain).map_err(|_| SpatialDemandError::CountOverflow {
            operation: "Godot demand radius",
        })?,
        u32::try_from(vertical_desired).map_err(|_| SpatialDemandError::CountOverflow {
            operation: "Godot demand radius",
        })?,
        u32::try_from(vertical_retain).map_err(|_| SpatialDemandError::CountOverflow {
            operation: "Godot demand radius",
        })?,
    ];
    if radii[1] < radii[0] {
        return Err(SpatialDemandError::RetainRadiusBelowDesired {
            axis: runen_spatial_demand::DemandAxis::Horizontal,
            desired: radii[0],
            retain: radii[1],
        });
    }
    if radii[3] < radii[2] {
        return Err(SpatialDemandError::RetainRadiusBelowDesired {
            axis: runen_spatial_demand::DemandAxis::Vertical,
            desired: radii[2],
            retain: radii[3],
        });
    }
    Ok(radii)
}

fn requested_budgets(requested: [i32; 2]) -> Result<StreamingBudgets, &'static str> {
    let max_load_requests_per_tick = usize::try_from(requested[0])
        .map_err(|_| "max_load_requests_per_tick must be non-negative")?;
    let max_unload_requests_per_tick = usize::try_from(requested[1])
        .map_err(|_| "max_unload_requests_per_tick must be non-negative")?;
    Ok(StreamingBudgets {
        max_load_requests_per_tick,
        max_unload_requests_per_tick,
    })
}

fn streaming_config_from_values(
    world_id: u16,
    chunk_edge_meters: f32,
    region_chunk_dims: [u32; 3],
    radii: [i32; 4],
    budgets: StreamingBudgets,
) -> Result<WorldStreamingConfig, SpatialDemandError> {
    let partition = partition_from_node_values(chunk_edge_meters, region_chunk_dims)?;
    requested_radii(radii)?;
    let mut config = WorldStreamingConfig::new(
        WorldId::new(world_id),
        partition,
        DemandLimits::default(),
        ADAPTER_STREAMING_CAPACITY,
    );
    config.budgets = budgets;
    Ok(config)
}

fn controller_has_runtime_records(controller: Option<&WorldStreamingController>) -> bool {
    controller.is_some_and(|controller| controller.records().next().is_some())
}

fn demand_changes_from_node_values(
    position: WorldPosition,
    requested: [i32; 4],
) -> Result<Vec<DemandSourceChange>, SpatialDemandError> {
    let radii = requested_radii(requested)?;
    let focus = DemandFocus::try_new(position, radii[0], radii[1], radii[2], radii[3])?;
    let snapshot = DemandSourceSnapshot::try_new(Some(focus), [])?;
    Ok(vec![DemandSourceChange::Replace {
        source_id: NODE_FOCUS_SOURCE,
        snapshot,
    }])
}

fn request_id_to_i64(request_id: StreamRequestId) -> Option<i64> {
    i64::try_from(request_id.get()).ok()
}

#[allow(dead_code)]
fn chunk_id_to_xyz(chunk_id: ChunkId) -> (i64, i64, i64) {
    (chunk_id.coord.x, chunk_id.coord.y, chunk_id.coord.z)
}

#[cfg(test)]
mod tests {
    use super::{
        ADAPTER_STREAMING_CAPACITY, NODE_FOCUS_SOURCE, controller_has_runtime_records,
        demand_changes_from_node_values, partition_from_node_values, request_id_to_i64,
        requested_budgets, requested_radii, requested_region_dims, requested_world_id,
    };
    use runen_spatial::{GridPartitionConfig, SpatialMathError, WorldId, WorldPosition};
    use runen_spatial_demand::{
        DemandAxis, DemandFocus, DemandLimits, DemandSourceChange, DemandSourceId,
        DemandSourceSnapshot, SpatialDemandError,
    };
    use runen_spatial_streaming::{
        ProviderEvent, ProviderEventKind, StreamRequest, StreamRequestId, StreamingBudgets,
        StreamingCapacity, StreamingTick, WorldStreamingConfig, WorldStreamingController,
    };

    #[test]
    fn world_id_rejects_out_of_range_values_without_repair() {
        assert_eq!(requested_world_id(0), Ok(0));
        assert_eq!(requested_world_id(i32::from(u16::MAX)), Ok(u16::MAX));
        for value in [-1, i32::from(u16::MAX) + 1, i32::MAX] {
            assert_eq!(
                requested_world_id(value),
                Err(SpatialMathError::CoordinateOutOfRange {
                    operation: "Godot world_id"
                })
            );
        }
    }

    #[test]
    fn partition_values_reject_invalid_edges_without_repairing_them() {
        for edge in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(matches!(
                partition_from_node_values(edge, [8, 8, 8]),
                Err(SpatialMathError::NonFiniteValue { .. })
                    | Err(SpatialMathError::NonPositiveValue { .. })
            ));
        }
    }

    #[test]
    fn requested_region_dimensions_reject_nonpositive_values() {
        for dimensions in [[0, 8, 8], [-1, 8, 8], [8, 0, 8], [8, 8, -1]] {
            assert!(matches!(
                requested_region_dims(dimensions),
                Err(SpatialMathError::NonPositiveValue { .. })
            ));
        }
    }

    #[test]
    fn valid_partition_update_builds_the_requested_configuration() {
        let config =
            partition_from_node_values(24.0, requested_region_dims([3, 4, 5]).unwrap()).unwrap();
        assert_eq!(config.chunk_edge_meters(), 24.0);
        assert_eq!(config.region_chunk_dims(), [3, 4, 5]);
    }

    #[test]
    fn demand_radii_reject_every_invalid_field_without_repair() {
        for radii in [[-1, 0, 0, 0], [0, -1, 0, 0], [0, 0, -1, 0], [0, 0, 0, -1]] {
            assert!(matches!(
                requested_radii(radii),
                Err(SpatialDemandError::CountOverflow {
                    operation: "Godot demand radius"
                })
            ));
        }
        assert_eq!(
            requested_radii([2, 1, 0, 0]),
            Err(SpatialDemandError::RetainRadiusBelowDesired {
                axis: DemandAxis::Horizontal,
                desired: 2,
                retain: 1,
            })
        );
        assert_eq!(
            requested_radii([0, 0, 2, 1]),
            Err(SpatialDemandError::RetainRadiusBelowDesired {
                axis: DemandAxis::Vertical,
                desired: 2,
                retain: 1,
            })
        );
        assert_eq!(requested_radii([2, 3, 4, 5]), Ok([2, 3, 4, 5]));
    }

    #[test]
    fn request_budgets_reject_negative_values_and_preserve_zero() {
        assert_eq!(
            requested_budgets([0, 3]),
            Ok(StreamingBudgets {
                max_load_requests_per_tick: 0,
                max_unload_requests_per_tick: 3,
            })
        );
        assert_eq!(
            requested_budgets([-1, 3]),
            Err("max_load_requests_per_tick must be non-negative")
        );
        assert_eq!(
            requested_budgets([3, -1]),
            Err("max_unload_requests_per_tick must be non-negative")
        );
    }

    #[test]
    fn adapter_transaction_is_a_complete_stable_source_replacement() {
        let position = WorldPosition::try_new(WorldId::new(23), [12.0, 0.0, -4.0]).unwrap();
        let changes = demand_changes_from_node_values(position, [2, 3, 4, 5]).unwrap();
        let changes = changes.iter().collect::<Vec<_>>();
        assert_eq!(changes.len(), 1);
        match changes[0] {
            DemandSourceChange::Replace {
                source_id,
                snapshot,
            } => {
                assert_eq!(*source_id, NODE_FOCUS_SOURCE);
                let focus = snapshot.focus().unwrap();
                assert_eq!(focus.position(), position);
                assert_eq!(focus.horizontal_desired_radius(), 2);
                assert_eq!(focus.horizontal_retain_radius(), 3);
                assert_eq!(focus.vertical_desired_radius(), 4);
                assert_eq!(focus.vertical_retain_radius(), 5);
                assert_eq!(snapshot.pins().count(), 0);
            }
            DemandSourceChange::Remove { .. } => panic!("adapter must publish a replacement"),
        }
    }

    #[test]
    fn adapter_streaming_capacity_is_explicit() {
        assert_eq!(ADAPTER_STREAMING_CAPACITY.max_tracked_records(), 1024);
        assert_eq!(ADAPTER_STREAMING_CAPACITY.max_in_flight_loads(), 4);
        assert_eq!(ADAPTER_STREAMING_CAPACITY.max_in_flight_unloads(), 4);
    }

    #[test]
    fn structural_rebuild_gate_blocks_every_runtime_record_kind() {
        let (active, request) = controller_with_load_request(StreamingCapacity::new(8, 1, 1));
        assert!(controller_has_runtime_records(Some(&active)));

        let (mut resident, resident_request) =
            controller_with_load_request(StreamingCapacity::new(8, 1, 1));
        resident
            .accept_provider_event(provider_event(
                resident_request,
                ProviderEventKind::Completed,
            ))
            .unwrap();
        assert!(controller_has_runtime_records(Some(&resident)));

        let (mut failed, failed_request) =
            controller_with_load_request(StreamingCapacity::new(8, 1, 1));
        failed
            .accept_provider_event(provider_event(failed_request, ProviderEventKind::Failed))
            .unwrap();
        assert!(controller_has_runtime_records(Some(&failed)));

        assert_eq!(active.pending_requests().count(), 1);
        assert_eq!(
            request.kind,
            runen_spatial_streaming::StreamRequestKind::Load
        );
    }

    #[test]
    fn planner_only_demand_does_not_block_structural_rebuild() {
        let capacity = StreamingCapacity::new(0, 0, 0);
        let partition = GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap();
        let config = WorldStreamingConfig::new(
            WorldId::new(7),
            partition,
            DemandLimits::default(),
            capacity,
        );
        let mut controller = WorldStreamingController::new(config);
        let focus = DemandFocus::try_new(
            WorldPosition::try_new(WorldId::new(7), [0.0, 0.0, 0.0]).unwrap(),
            0,
            0,
            0,
            0,
        )
        .unwrap();
        let snapshot = DemandSourceSnapshot::try_new(Some(focus), []).unwrap();
        let output = controller
            .tick(StreamingTick::from_demand_changes([
                DemandSourceChange::Replace {
                    source_id: DemandSourceId::new(0),
                    snapshot,
                },
            ]))
            .unwrap();

        assert!(output.requests.is_empty());
        assert_eq!(controller.effective_demand().len(), 1);
        assert!(!controller_has_runtime_records(Some(&controller)));
    }

    #[test]
    fn request_id_translation_is_exact_and_fallible() {
        let godot_max = u64::try_from(i64::MAX).unwrap();
        let maximum = StreamRequestId::try_new(godot_max).unwrap();
        assert_eq!(request_id_to_i64(maximum), Some(i64::MAX));

        let too_large = StreamRequestId::try_new(godot_max + 1).unwrap();
        assert_eq!(request_id_to_i64(too_large), None);
    }

    fn controller_with_load_request(
        capacity: StreamingCapacity,
    ) -> (WorldStreamingController, StreamRequest) {
        let partition = GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap();
        let mut config = WorldStreamingConfig::new(
            WorldId::new(7),
            partition,
            DemandLimits::default(),
            capacity,
        );
        config.budgets = StreamingBudgets {
            max_load_requests_per_tick: 1,
            max_unload_requests_per_tick: 1,
        };
        let mut controller = WorldStreamingController::new(config);
        let focus = DemandFocus::try_new(
            WorldPosition::try_new(WorldId::new(7), [0.0, 0.0, 0.0]).unwrap(),
            0,
            0,
            0,
            0,
        )
        .unwrap();
        let snapshot = DemandSourceSnapshot::try_new(Some(focus), []).unwrap();
        let request = controller
            .tick(StreamingTick::from_demand_changes([
                DemandSourceChange::Replace {
                    source_id: DemandSourceId::new(0),
                    snapshot,
                },
            ]))
            .unwrap()
            .requests[0];
        (controller, request)
    }

    fn provider_event(request: StreamRequest, kind: ProviderEventKind) -> ProviderEvent {
        ProviderEvent {
            request_id: request.request_id,
            chunk_id: request.chunk_id,
            kind,
        }
    }
}
