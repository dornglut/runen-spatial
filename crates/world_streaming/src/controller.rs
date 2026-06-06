use crate::error::WorldStreamingError;
use crate::events::{
    ProviderEvent, ProviderEventKind, WorldStreamingEvent, WorldStreamingEventKind,
};
use crate::lifecycle::ChunkLifecycleState;
use crate::priority::{ChunkPriority, distance_squared};
use crate::request::{StreamRequest, StreamRequestId, StreamRequestKind};
use chunking::{ChunkSetDiff, ChunkStreamer, ChunkStreamingConfig, StreamingFocus};
use spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
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
    pub chunking: ChunkStreamingConfig,
    pub budgets: StreamingBudgets,
}

impl WorldStreamingConfig {
    pub fn new(
        world_id: WorldId,
        partition: GridPartitionConfig,
        chunking: ChunkStreamingConfig,
    ) -> Self {
        Self {
            world_id,
            partition,
            chunking,
            budgets: StreamingBudgets::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StreamingTick {
    pub focus: Option<StreamingFocus>,
}

impl StreamingTick {
    pub fn from_focus(focus: StreamingFocus) -> Self {
        Self { focus: Some(focus) }
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
    pub priority: ChunkPriority,
    pub active_request_id: Option<StreamRequestId>,
    pub active_request_kind: Option<StreamRequestKind>,
}

impl ChunkRuntimeRecord {
    fn new(chunk_id: ChunkId) -> Self {
        Self {
            chunk_id,
            state: ChunkLifecycleState::Absent,
            desired: false,
            priority: ChunkPriority::default(),
            active_request_id: None,
            active_request_kind: None,
        }
    }
}

pub struct WorldStreamingController {
    world_id: WorldId,
    streamer: ChunkStreamer,
    budgets: StreamingBudgets,
    records: BTreeMap<ChunkId, ChunkRuntimeRecord>,
    pending_requests: BTreeMap<StreamRequestId, StreamRequest>,
    next_request_id: u64,
}

impl WorldStreamingController {
    pub fn new(config: WorldStreamingConfig) -> Self {
        Self {
            world_id: config.world_id,
            streamer: ChunkStreamer::new(config.partition, config.chunking),
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

    pub fn chunking_config(&self) -> ChunkStreamingConfig {
        self.streamer.config()
    }

    pub fn set_chunking_config(&mut self, config: ChunkStreamingConfig) {
        self.streamer.set_config(config);
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

    pub fn tick(&mut self, tick: StreamingTick) -> StreamingTickOutput {
        let mut events = Vec::new();

        if let Some(focus) = tick.focus {
            let center = self.streamer.center_chunk_for_focus(focus);
            let diff = self.streamer.update_focus(focus);
            self.apply_chunk_diff(center, diff, &mut events);
        }
        self.requeue_failed_records(&mut events);

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

        StreamingTickOutput { requests, events }
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
                events.push(WorldStreamingEvent::new(
                    record.chunk_id,
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
                events.push(WorldStreamingEvent::new(
                    record.chunk_id,
                    WorldStreamingEventKind::ProviderCompleted,
                ));
                events.push(WorldStreamingEvent::new(
                    record.chunk_id,
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
                record.state = ChunkLifecycleState::Failed;
                events.push(WorldStreamingEvent::new(
                    record.chunk_id,
                    WorldStreamingEventKind::ProviderFailed,
                ));
            }
            (
                StreamRequestKind::Unload,
                ProviderEventKind::Started,
                ChunkLifecycleState::UnloadRequested,
            ) => {
                record.state = ChunkLifecycleState::Unloading;
                events.push(WorldStreamingEvent::new(
                    record.chunk_id,
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
                events.push(WorldStreamingEvent::new(
                    record.chunk_id,
                    WorldStreamingEventKind::ProviderCompleted,
                ));
                events.push(WorldStreamingEvent::new(
                    record.chunk_id,
                    WorldStreamingEventKind::Unloaded,
                ));
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
                events.push(WorldStreamingEvent::new(
                    record.chunk_id,
                    WorldStreamingEventKind::ProviderFailed,
                ));
            }
            (_, ProviderEventKind::Cancelled, _) => {
                self.pending_requests.remove(&event.request_id);
                record.active_request_id = None;
                record.active_request_kind = None;
                record.state = if record.desired {
                    ChunkLifecycleState::LoadQueued
                } else {
                    ChunkLifecycleState::Absent
                };
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

    fn apply_chunk_diff(
        &mut self,
        center: ChunkCoord3,
        diff: ChunkSetDiff,
        events: &mut Vec<WorldStreamingEvent>,
    ) {
        for (rank, coord) in diff.entered.into_iter().enumerate() {
            let chunk_id = ChunkId::new(self.world_id, coord);
            let priority = ChunkPriority::new(rank as u32, distance_squared(coord, center));
            self.mark_desired(chunk_id, priority, events);
        }

        for (rank, coord) in diff.exited.into_iter().enumerate() {
            let chunk_id = ChunkId::new(self.world_id, coord);
            let priority = ChunkPriority::new(rank as u32, distance_squared(coord, center));
            self.mark_undesired(chunk_id, priority, events);
        }
    }

    fn mark_desired(
        &mut self,
        chunk_id: ChunkId,
        priority: ChunkPriority,
        events: &mut Vec<WorldStreamingEvent>,
    ) {
        let record = self
            .records
            .entry(chunk_id)
            .or_insert_with(|| ChunkRuntimeRecord::new(chunk_id));
        record.desired = true;
        record.priority = priority;

        if matches!(
            record.state,
            ChunkLifecycleState::Absent | ChunkLifecycleState::Failed
        ) {
            record.state = ChunkLifecycleState::LoadQueued;
            events.push(WorldStreamingEvent::new(
                chunk_id,
                WorldStreamingEventKind::LoadQueued,
            ));
        }
    }

    fn mark_undesired(
        &mut self,
        chunk_id: ChunkId,
        priority: ChunkPriority,
        events: &mut Vec<WorldStreamingEvent>,
    ) {
        let Some(record) = self.records.get_mut(&chunk_id) else {
            return;
        };
        record.desired = false;
        record.priority = priority;

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
            .map(|record| (record.priority, record.chunk_id))
            .collect::<Vec<_>>();
        candidates.sort_by_key(|(priority, chunk_id)| (*priority, *chunk_id));

        for (_, chunk_id) in candidates.into_iter().take(budget) {
            let request_id = self.next_request_id();
            let Some(record) = self.records.get_mut(&chunk_id) else {
                continue;
            };
            record.state = match request_kind {
                StreamRequestKind::Load => ChunkLifecycleState::LoadRequested,
                StreamRequestKind::Unload => ChunkLifecycleState::UnloadRequested,
                StreamRequestKind::CancelLoad | StreamRequestKind::CancelUnload => record.state,
            };
            record.active_request_id = Some(request_id);
            record.active_request_kind = Some(request_kind);

            let request = StreamRequest {
                request_id,
                chunk_id,
                kind: request_kind,
                priority: record.priority,
            };
            self.pending_requests.insert(request_id, request);
            requests.push(request);
            events.push(WorldStreamingEvent::new(
                chunk_id,
                match request_kind {
                    StreamRequestKind::Load => WorldStreamingEventKind::LoadRequested,
                    StreamRequestKind::Unload => WorldStreamingEventKind::UnloadRequested,
                    StreamRequestKind::CancelLoad => WorldStreamingEventKind::LoadRequestCancelled,
                    StreamRequestKind::CancelUnload => {
                        WorldStreamingEventKind::UnloadRequestCancelled
                    }
                },
            ));
        }
    }

    fn requeue_failed_records(&mut self, events: &mut Vec<WorldStreamingEvent>) {
        let failed = self
            .records
            .iter()
            .filter_map(|(chunk_id, record)| {
                (record.state == ChunkLifecycleState::Failed).then_some(*chunk_id)
            })
            .collect::<Vec<_>>();

        for chunk_id in failed {
            let Some(record) = self.records.get_mut(&chunk_id) else {
                continue;
            };
            record.state = if record.desired {
                ChunkLifecycleState::LoadQueued
            } else {
                ChunkLifecycleState::Absent
            };
            if record.state == ChunkLifecycleState::LoadQueued {
                events.push(WorldStreamingEvent::new(
                    chunk_id,
                    WorldStreamingEventKind::LoadQueued,
                ));
            }
        }
    }

    fn next_request_id(&mut self) -> StreamRequestId {
        let request_id = StreamRequestId(self.next_request_id);
        self.next_request_id = self.next_request_id.saturating_add(1);
        request_id
    }
}
