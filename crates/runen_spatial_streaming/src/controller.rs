use crate::error::WorldStreamingError;
use crate::events::{
    ProviderEvent, ProviderEventKind, WorldStreamingEvent, WorldStreamingEventKind,
};
use crate::lifecycle::ChunkLifecycleState;
use crate::request::{StreamRequest, StreamRequestId, StreamRequestKind};
use runen_spatial::{ChunkId, GridPartitionConfig, WorldId};
use runen_spatial_demand::{
    DemandLimits, DemandRank, DemandSourceChange, SpatialDemandDelta, SpatialDemandPlanner,
};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StreamingBudgets {
    pub max_load_requests_per_tick: usize,
    pub max_unload_requests_per_tick: usize,
}

impl Default for StreamingBudgets {
    fn default() -> Self {
        Self {
            max_load_requests_per_tick: 4,
            max_unload_requests_per_tick: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldStreamingConfig {
    pub world_id: WorldId,
    pub partition: GridPartitionConfig,
    pub demand_limits: DemandLimits,
    pub budgets: StreamingBudgets,
}

impl WorldStreamingConfig {
    pub fn new(
        world_id: WorldId,
        partition: GridPartitionConfig,
        demand_limits: DemandLimits,
    ) -> Self {
        Self {
            world_id,
            partition,
            demand_limits,
            budgets: StreamingBudgets::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StreamingTick {
    demand_changes: Vec<DemandSourceChange>,
}

impl StreamingTick {
    pub const fn without_demand_changes() -> Self {
        Self {
            demand_changes: Vec::new(),
        }
    }

    pub fn from_demand_changes(changes: impl IntoIterator<Item = DemandSourceChange>) -> Self {
        Self {
            demand_changes: changes.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StreamingTickOutput {
    pub requests: Vec<StreamRequest>,
    pub events: Vec<WorldStreamingEvent>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChunkRuntimeRecord {
    pub chunk_id: ChunkId,
    pub state: ChunkLifecycleState,
    pub desired: bool,
    pub rank: DemandRank,
    pub active_request_id: Option<StreamRequestId>,
    pub active_request_kind: Option<StreamRequestKind>,
}

impl ChunkRuntimeRecord {
    fn new(chunk_id: ChunkId) -> Self {
        Self {
            chunk_id,
            state: ChunkLifecycleState::Absent,
            desired: false,
            rank: DemandRank::default(),
            active_request_id: None,
            active_request_kind: None,
        }
    }
}

pub struct WorldStreamingController {
    world_id: WorldId,
    planner: SpatialDemandPlanner,
    budgets: StreamingBudgets,
    records: BTreeMap<ChunkId, ChunkRuntimeRecord>,
    pending_requests: BTreeMap<StreamRequestId, StreamRequest>,
    next_request_id: u64,
}

impl WorldStreamingController {
    pub fn new(config: WorldStreamingConfig) -> Self {
        Self {
            world_id: config.world_id,
            planner: SpatialDemandPlanner::new(
                config.world_id,
                config.partition,
                config.demand_limits,
            ),
            budgets: config.budgets,
            records: BTreeMap::new(),
            pending_requests: BTreeMap::new(),
            next_request_id: 1,
        }
    }

    pub fn world_id(&self) -> WorldId {
        self.world_id
    }

    pub fn budgets(&self) -> StreamingBudgets {
        self.budgets
    }

    pub fn set_budgets(&mut self, budgets: StreamingBudgets) {
        self.budgets = budgets;
    }

    pub fn demand_limits(&self) -> DemandLimits {
        self.planner.limits()
    }
    pub fn effective_demand(&self) -> &runen_spatial_demand::EffectiveDemandSnapshot {
        self.planner.effective_snapshot()
    }

    pub fn records(&self) -> impl Iterator<Item = &ChunkRuntimeRecord> {
        self.records.values()
    }

    pub fn record(&self, chunk_id: ChunkId) -> Option<&ChunkRuntimeRecord> {
        self.records.get(&chunk_id)
    }

    pub fn pending_requests(&self) -> impl Iterator<Item = &StreamRequest> {
        self.pending_requests.values()
    }

    pub fn tick(
        &mut self,
        tick: StreamingTick,
    ) -> Result<StreamingTickOutput, WorldStreamingError> {
        let mut events = Vec::new();

        if !tick.demand_changes.is_empty() {
            let delta = self
                .planner
                .apply_changes(tick.demand_changes)
                .map_err(WorldStreamingError::SpatialDemand)?;
            self.apply_demand_delta(&delta, &mut events);
        }

        let mut requests = Vec::new();
        self.emit_requests_for_state(
            ChunkLifecycleState::LoadQueued,
            StreamRequestKind::Load,
            self.budgets.max_load_requests_per_tick,
            &mut requests,
            &mut events,
        );
        self.emit_requests_for_state(
            ChunkLifecycleState::UnloadQueued,
            StreamRequestKind::Unload,
            self.budgets.max_unload_requests_per_tick,
            &mut requests,
            &mut events,
        );

        Ok(StreamingTickOutput { requests, events })
    }

    pub fn accept_provider_event(
        &mut self,
        event: ProviderEvent,
    ) -> Result<Vec<WorldStreamingEvent>, WorldStreamingError> {
        let Some(request) = self.pending_requests.get(&event.request_id).copied() else {
            return Err(WorldStreamingError::UnknownRequest {
                request_id: event.request_id,
            });
        };

        if request.chunk_id != event.chunk_id {
            return Err(WorldStreamingError::RequestChunkMismatch {
                request_id: event.request_id,
                expected: request.chunk_id,
                actual: event.chunk_id,
            });
        }

        let Some(record) = self.records.get_mut(&event.chunk_id) else {
            return Err(WorldStreamingError::UnknownRequest {
                request_id: event.request_id,
            });
        };

        let mut events = Vec::new();
        match (request.kind, event.kind, record.state) {
            (
                StreamRequestKind::Load,
                ProviderEventKind::Started,
                ChunkLifecycleState::LoadRequested,
            ) => {
                record.state = ChunkLifecycleState::Loading;
                events.push(WorldStreamingEvent::with_request(
                    record.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderStarted,
                ));
            }
            (
                StreamRequestKind::Load,
                ProviderEventKind::Completed,
                ChunkLifecycleState::LoadRequested | ChunkLifecycleState::Loading,
            ) => {
                self.pending_requests.remove(&event.request_id);
                record.active_request_id = None;
                record.active_request_kind = None;
                record.state = ChunkLifecycleState::Resident;
                events.push(WorldStreamingEvent::with_request(
                    record.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderCompleted,
                ));
                events.push(WorldStreamingEvent::with_request(
                    record.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::Resident,
                ));
                if !record.desired {
                    record.state = ChunkLifecycleState::UnloadQueued;
                    events.push(WorldStreamingEvent::new(
                        record.chunk_id,
                        WorldStreamingEventKind::UnloadQueued,
                    ));
                }
            }
            (
                StreamRequestKind::Load,
                ProviderEventKind::Failed,
                ChunkLifecycleState::LoadRequested | ChunkLifecycleState::Loading,
            ) => {
                self.pending_requests.remove(&event.request_id);
                record.active_request_id = None;
                record.active_request_kind = None;
                record.state = if record.desired {
                    ChunkLifecycleState::Failed
                } else {
                    ChunkLifecycleState::Absent
                };
                events.push(WorldStreamingEvent::with_request(
                    record.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderFailed,
                ));
            }
            (
                StreamRequestKind::Unload,
                ProviderEventKind::Started,
                ChunkLifecycleState::UnloadRequested,
            ) => {
                record.state = ChunkLifecycleState::Unloading;
                events.push(WorldStreamingEvent::with_request(
                    record.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderStarted,
                ));
            }
            (
                StreamRequestKind::Unload,
                ProviderEventKind::Completed,
                ChunkLifecycleState::UnloadRequested | ChunkLifecycleState::Unloading,
            ) => {
                self.pending_requests.remove(&event.request_id);
                record.active_request_id = None;
                record.active_request_kind = None;
                record.state = ChunkLifecycleState::Absent;
                events.push(WorldStreamingEvent::with_request(
                    record.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderCompleted,
                ));
                events.push(WorldStreamingEvent::with_request(
                    record.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::Unloaded,
                ));
                if record.desired {
                    record.state = ChunkLifecycleState::LoadQueued;
                    events.push(WorldStreamingEvent::new(
                        record.chunk_id,
                        WorldStreamingEventKind::LoadQueued,
                    ));
                }
            }
            (
                StreamRequestKind::Unload,
                ProviderEventKind::Failed,
                ChunkLifecycleState::UnloadRequested | ChunkLifecycleState::Unloading,
            ) => {
                self.pending_requests.remove(&event.request_id);
                record.active_request_id = None;
                record.active_request_kind = None;
                record.state = ChunkLifecycleState::Failed;
                events.push(WorldStreamingEvent::with_request(
                    record.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderFailed,
                ));
            }
            _ => {
                return Err(WorldStreamingError::InvalidProviderEvent {
                    request_id: event.request_id,
                    request_kind: request.kind,
                    event_kind: event.kind,
                    state: record.state,
                });
            }
        }

        Ok(events)
    }

    pub fn fail_resident_chunk(
        &mut self,
        chunk_id: ChunkId,
    ) -> Result<WorldStreamingEvent, WorldStreamingError> {
        let Some(record) = self.records.get_mut(&chunk_id) else {
            return Err(WorldStreamingError::UnknownChunk { chunk_id });
        };
        if record.state != ChunkLifecycleState::Resident {
            return Err(WorldStreamingError::InvalidResidentFailure {
                chunk_id,
                state: record.state,
            });
        }

        record.state = ChunkLifecycleState::Failed;
        Ok(WorldStreamingEvent::new(
            chunk_id,
            WorldStreamingEventKind::ProviderFailed,
        ))
    }

    pub fn retry_failed_chunk(
        &mut self,
        chunk_id: ChunkId,
    ) -> Result<WorldStreamingEvent, WorldStreamingError> {
        let Some(record) = self.records.get_mut(&chunk_id) else {
            return Err(WorldStreamingError::UnknownChunk { chunk_id });
        };
        if record.state != ChunkLifecycleState::Failed || !record.desired {
            return Err(WorldStreamingError::InvalidFailedRetry {
                chunk_id,
                state: record.state,
                desired: record.desired,
            });
        }

        record.state = ChunkLifecycleState::LoadQueued;
        let event = WorldStreamingEvent::new(chunk_id, WorldStreamingEventKind::LoadQueued);
        Ok(event)
    }

    fn apply_demand_delta(
        &mut self,
        delta: &SpatialDemandDelta,
        events: &mut Vec<WorldStreamingEvent>,
    ) {
        for chunk in delta.entered() {
            self.mark_desired(chunk.chunk_id(), chunk.rank(), events);
        }
        for chunk in delta.updated() {
            self.refresh_priority(chunk.chunk_id(), chunk.rank());
        }
        for chunk in delta.exited() {
            self.mark_undesired(chunk.chunk_id(), chunk.rank(), events);
        }
    }

    fn mark_desired(
        &mut self,
        chunk_id: ChunkId,
        rank: DemandRank,
        events: &mut Vec<WorldStreamingEvent>,
    ) {
        let record = self
            .records
            .entry(chunk_id)
            .or_insert_with(|| ChunkRuntimeRecord::new(chunk_id));
        record.desired = true;
        record.rank = rank;

        match record.state {
            ChunkLifecycleState::Absent => {
                record.state = ChunkLifecycleState::LoadQueued;
                events.push(WorldStreamingEvent::new(
                    chunk_id,
                    WorldStreamingEventKind::LoadQueued,
                ));
            }
            ChunkLifecycleState::UnloadQueued => {
                record.state = ChunkLifecycleState::Resident;
            }
            ChunkLifecycleState::Failed
            | ChunkLifecycleState::LoadQueued
            | ChunkLifecycleState::LoadRequested
            | ChunkLifecycleState::Loading
            | ChunkLifecycleState::Resident
            | ChunkLifecycleState::UnloadRequested
            | ChunkLifecycleState::Unloading => {}
        }
    }

    fn mark_undesired(
        &mut self,
        chunk_id: ChunkId,
        rank: DemandRank,
        events: &mut Vec<WorldStreamingEvent>,
    ) {
        let Some(record) = self.records.get_mut(&chunk_id) else {
            return;
        };
        record.desired = false;
        record.rank = rank;

        match record.state {
            ChunkLifecycleState::LoadQueued | ChunkLifecycleState::Failed => {
                record.state = ChunkLifecycleState::Absent;
            }
            ChunkLifecycleState::Resident => {
                record.state = ChunkLifecycleState::UnloadQueued;
                events.push(WorldStreamingEvent::new(
                    chunk_id,
                    WorldStreamingEventKind::UnloadQueued,
                ));
            }
            _ => {}
        }
    }

    fn refresh_priority(&mut self, chunk_id: ChunkId, rank: DemandRank) {
        if let Some(record) = self.records.get_mut(&chunk_id) {
            record.rank = rank;
        }
    }

    fn emit_requests_for_state(
        &mut self,
        state: ChunkLifecycleState,
        request_kind: StreamRequestKind,
        budget: usize,
        requests: &mut Vec<StreamRequest>,
        events: &mut Vec<WorldStreamingEvent>,
    ) {
        let mut candidates = self
            .records
            .values()
            .filter(|record| record.state == state)
            .map(|record| (record.rank, record.chunk_id))
            .collect::<Vec<_>>();
        candidates.sort_by_key(|(rank, chunk_id)| (*rank, *chunk_id));

        for (_, chunk_id) in candidates.into_iter().take(budget) {
            let request_id = self.next_request_id();
            let Some(record) = self.records.get_mut(&chunk_id) else {
                continue;
            };
            record.state = match request_kind {
                StreamRequestKind::Load => ChunkLifecycleState::LoadRequested,
                StreamRequestKind::Unload => ChunkLifecycleState::UnloadRequested,
            };
            record.active_request_id = Some(request_id);
            record.active_request_kind = Some(request_kind);

            let request = StreamRequest {
                request_id,
                chunk_id,
                kind: request_kind,
                rank: record.rank,
            };
            self.pending_requests.insert(request_id, request);
            requests.push(request);
            events.push(WorldStreamingEvent::with_request(
                chunk_id,
                request_id,
                match request_kind {
                    StreamRequestKind::Load => WorldStreamingEventKind::LoadRequested,
                    StreamRequestKind::Unload => WorldStreamingEventKind::UnloadRequested,
                },
            ));
        }
    }

    fn next_request_id(&mut self) -> StreamRequestId {
        let request_id = StreamRequestId(self.next_request_id);
        self.next_request_id = self.next_request_id.saturating_add(1);
        request_id
    }
}
