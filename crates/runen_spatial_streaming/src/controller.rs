use crate::error::WorldStreamingError;
use crate::events::{
    ProviderEvent, ProviderEventKind, WorldStreamingEvent, WorldStreamingEventKind,
};
use crate::lifecycle::{ChunkAvailability, ChunkOperation};
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
    pub request_id_exhausted: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChunkRuntimeRecord {
    chunk_id: ChunkId,
    desired: bool,
    rank: DemandRank,
    availability: ChunkAvailability,
    operation: ChunkOperation,
    blocking_failure: Option<StreamRequestKind>,
}

impl ChunkRuntimeRecord {
    fn new(chunk_id: ChunkId) -> Self {
        Self {
            chunk_id,
            desired: false,
            rank: DemandRank::default(),
            availability: ChunkAvailability::Absent,
            operation: ChunkOperation::Idle,
            blocking_failure: None,
        }
    }

    pub const fn chunk_id(&self) -> ChunkId {
        self.chunk_id
    }

    pub const fn desired(&self) -> bool {
        self.desired
    }

    pub const fn rank(&self) -> DemandRank {
        self.rank
    }

    pub const fn availability(&self) -> ChunkAvailability {
        self.availability
    }

    pub const fn operation(&self) -> ChunkOperation {
        self.operation
    }

    pub const fn blocking_failure(&self) -> Option<StreamRequestKind> {
        self.blocking_failure
    }
}

pub struct WorldStreamingController {
    world_id: WorldId,
    planner: SpatialDemandPlanner,
    budgets: StreamingBudgets,
    records: BTreeMap<ChunkId, ChunkRuntimeRecord>,
    pending_requests: BTreeMap<StreamRequestId, ChunkId>,
    next_request_id: Option<StreamRequestId>,
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
            next_request_id: StreamRequestId::try_new(1),
        }
    }

    pub const fn world_id(&self) -> WorldId {
        self.world_id
    }

    pub const fn budgets(&self) -> StreamingBudgets {
        self.budgets
    }

    pub fn set_budgets(&mut self, budgets: StreamingBudgets) {
        self.budgets = budgets;
    }

    pub const fn demand_limits(&self) -> DemandLimits {
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
        self.pending_requests
            .iter()
            .map(move |(request_id, chunk_id)| {
                let record = self
                    .records
                    .get(chunk_id)
                    .expect("pending request must reference a live record");
                let request = record
                    .operation
                    .active_request()
                    .expect("pending request must be owned by the active operation");
                debug_assert_eq!(request.request_id, *request_id);
                request
            })
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
            self.prune_neutral_records();
        }

        let load_candidates = self.queued_candidates(
            StreamRequestKind::Load,
            self.budgets.max_load_requests_per_tick,
        );
        let unload_candidates = self.queued_candidates(
            StreamRequestKind::Unload,
            self.budgets.max_unload_requests_per_tick,
        );
        let request_count = load_candidates.len() + unload_candidates.len();
        let Some(request_ids) = self.reserve_request_ids(request_count) else {
            return Ok(StreamingTickOutput {
                requests: Vec::new(),
                events,
                request_id_exhausted: true,
            });
        };

        let (load_request_ids, unload_request_ids) = request_ids.split_at(load_candidates.len());
        let mut requests = Vec::with_capacity(request_count);
        self.issue_candidates(
            StreamRequestKind::Load,
            &load_candidates,
            load_request_ids,
            &mut requests,
            &mut events,
        );
        self.issue_candidates(
            StreamRequestKind::Unload,
            &unload_candidates,
            unload_request_ids,
            &mut requests,
            &mut events,
        );

        Ok(StreamingTickOutput {
            requests,
            events,
            request_id_exhausted: false,
        })
    }

    pub fn accept_provider_event(
        &mut self,
        event: ProviderEvent,
    ) -> Result<Vec<WorldStreamingEvent>, WorldStreamingError> {
        let Some(expected_chunk) = self.pending_requests.get(&event.request_id).copied() else {
            return Err(WorldStreamingError::UnknownRequest {
                request_id: event.request_id,
            });
        };
        if expected_chunk != event.chunk_id {
            return Err(WorldStreamingError::RequestChunkMismatch {
                request_id: event.request_id,
                expected: expected_chunk,
                actual: event.chunk_id,
            });
        }

        let Some(record) = self.records.get(&event.chunk_id) else {
            return Err(WorldStreamingError::UnknownRequest {
                request_id: event.request_id,
            });
        };
        let availability = record.availability;
        let operation = record.operation;
        let Some(active_request) = operation.active_request().copied() else {
            return Err(WorldStreamingError::InvalidProviderEvent {
                request_id: event.request_id,
                event_kind: event.kind,
                availability,
                operation,
            });
        };
        if active_request.request_id != event.request_id {
            return Err(WorldStreamingError::InvalidProviderEvent {
                request_id: event.request_id,
                event_kind: event.kind,
                availability,
                operation,
            });
        }

        let mut events = Vec::new();
        match (operation, event.kind, availability) {
            (
                ChunkOperation::LoadRequested(request),
                ProviderEventKind::Started,
                ChunkAvailability::Absent,
            ) => {
                self.records.get_mut(&event.chunk_id).unwrap().operation =
                    ChunkOperation::Loading(request);
                events.push(WorldStreamingEvent::with_request(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderStarted,
                ));
            }
            (
                ChunkOperation::LoadRequested(_) | ChunkOperation::Loading(_),
                ProviderEventKind::Completed,
                ChunkAvailability::Absent,
            ) => {
                self.finish_request(event.request_id, event.chunk_id);
                let record = self.records.get_mut(&event.chunk_id).unwrap();
                record.availability = ChunkAvailability::Resident;
                record.operation = ChunkOperation::Idle;
                record.blocking_failure = None;
                events.push(WorldStreamingEvent::with_request(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderCompleted,
                ));
                events.push(WorldStreamingEvent::with_request(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::Resident,
                ));
                Self::reconcile_idle_record(record, &mut events);
            }
            (
                ChunkOperation::LoadRequested(_) | ChunkOperation::Loading(_),
                ProviderEventKind::Failed,
                ChunkAvailability::Absent,
            ) => {
                self.finish_request(event.request_id, event.chunk_id);
                let record = self.records.get_mut(&event.chunk_id).unwrap();
                record.operation = ChunkOperation::Idle;
                record.blocking_failure = record.desired.then_some(StreamRequestKind::Load);
                events.push(WorldStreamingEvent::with_request(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderFailed,
                ));
                Self::reconcile_idle_record(record, &mut events);
            }
            (
                ChunkOperation::UnloadRequested(request),
                ProviderEventKind::Started,
                ChunkAvailability::Resident,
            ) => {
                self.records.get_mut(&event.chunk_id).unwrap().operation =
                    ChunkOperation::Unloading(request);
                events.push(WorldStreamingEvent::with_request(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderStarted,
                ));
            }
            (
                ChunkOperation::UnloadRequested(_) | ChunkOperation::Unloading(_),
                ProviderEventKind::Completed,
                ChunkAvailability::Resident,
            ) => {
                self.finish_request(event.request_id, event.chunk_id);
                let record = self.records.get_mut(&event.chunk_id).unwrap();
                record.availability = ChunkAvailability::Absent;
                record.operation = ChunkOperation::Idle;
                record.blocking_failure = None;
                events.push(WorldStreamingEvent::with_request(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderCompleted,
                ));
                events.push(WorldStreamingEvent::with_request(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::Unloaded,
                ));
                Self::reconcile_idle_record(record, &mut events);
            }
            (
                ChunkOperation::UnloadRequested(_) | ChunkOperation::Unloading(_),
                ProviderEventKind::Failed,
                ChunkAvailability::Resident,
            ) => {
                self.finish_request(event.request_id, event.chunk_id);
                let record = self.records.get_mut(&event.chunk_id).unwrap();
                record.operation = ChunkOperation::Idle;
                record.blocking_failure = (!record.desired).then_some(StreamRequestKind::Unload);
                events.push(WorldStreamingEvent::with_request(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderFailed,
                ));
                Self::reconcile_idle_record(record, &mut events);
            }
            _ => {
                return Err(WorldStreamingError::InvalidProviderEvent {
                    request_id: event.request_id,
                    event_kind: event.kind,
                    availability,
                    operation,
                });
            }
        }

        self.prune_neutral_records();
        Ok(events)
    }

    pub fn retry_blocking_failure(
        &mut self,
        chunk_id: ChunkId,
    ) -> Result<WorldStreamingEvent, WorldStreamingError> {
        let Some(record) = self.records.get_mut(&chunk_id) else {
            return Err(WorldStreamingError::UnknownChunk { chunk_id });
        };

        let next_operation = match (
            record.desired,
            record.availability,
            record.operation,
            record.blocking_failure,
        ) {
            (
                true,
                ChunkAvailability::Absent,
                ChunkOperation::Idle,
                Some(StreamRequestKind::Load),
            ) => ChunkOperation::LoadQueued,
            (
                false,
                ChunkAvailability::Resident,
                ChunkOperation::Idle,
                Some(StreamRequestKind::Unload),
            ) => ChunkOperation::UnloadQueued,
            _ => {
                return Err(WorldStreamingError::InvalidBlockingFailureRetry {
                    chunk_id,
                    desired: record.desired,
                    availability: record.availability,
                    operation: record.operation,
                    blocking_failure: record.blocking_failure,
                });
            }
        };

        let kind = record.blocking_failure.unwrap();
        record.blocking_failure = None;
        record.operation = next_operation;
        Ok(WorldStreamingEvent::new(
            chunk_id,
            match kind {
                StreamRequestKind::Load => WorldStreamingEventKind::LoadQueued,
                StreamRequestKind::Unload => WorldStreamingEventKind::UnloadQueued,
            },
        ))
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
            self.refresh_rank(chunk.chunk_id(), chunk.rank());
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
        if record.operation == ChunkOperation::UnloadQueued {
            record.operation = ChunkOperation::Idle;
        }
        Self::reconcile_idle_record(record, events);
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
        if record.operation == ChunkOperation::LoadQueued {
            record.operation = ChunkOperation::Idle;
        }
        Self::reconcile_idle_record(record, events);
    }

    fn refresh_rank(&mut self, chunk_id: ChunkId, rank: DemandRank) {
        if let Some(record) = self.records.get_mut(&chunk_id) {
            record.rank = rank;
        }
    }

    fn reconcile_idle_record(
        record: &mut ChunkRuntimeRecord,
        events: &mut Vec<WorldStreamingEvent>,
    ) {
        if record.operation != ChunkOperation::Idle {
            return;
        }

        let target_satisfied = match record.availability {
            ChunkAvailability::Absent => !record.desired,
            ChunkAvailability::Resident => record.desired,
        };
        if target_satisfied {
            record.blocking_failure = None;
            return;
        }

        let required_kind = if record.desired {
            StreamRequestKind::Load
        } else {
            StreamRequestKind::Unload
        };
        if let Some(failure) = record.blocking_failure {
            if failure == required_kind {
                return;
            }
            record.blocking_failure = None;
        }

        record.operation = match required_kind {
            StreamRequestKind::Load => ChunkOperation::LoadQueued,
            StreamRequestKind::Unload => ChunkOperation::UnloadQueued,
        };
        events.push(WorldStreamingEvent::new(
            record.chunk_id,
            match required_kind {
                StreamRequestKind::Load => WorldStreamingEventKind::LoadQueued,
                StreamRequestKind::Unload => WorldStreamingEventKind::UnloadQueued,
            },
        ));
    }

    fn queued_candidates(&self, kind: StreamRequestKind, budget: usize) -> Vec<ChunkId> {
        let mut candidates = self
            .records
            .values()
            .filter(|record| {
                matches!(
                    (kind, record.operation),
                    (StreamRequestKind::Load, ChunkOperation::LoadQueued)
                        | (StreamRequestKind::Unload, ChunkOperation::UnloadQueued)
                )
            })
            .map(|record| (record.rank, record.chunk_id))
            .collect::<Vec<_>>();
        candidates.sort_by_key(|(rank, chunk_id)| (*rank, *chunk_id));
        candidates.truncate(budget);
        candidates
            .into_iter()
            .map(|(_, chunk_id)| chunk_id)
            .collect()
    }

    fn reserve_request_ids(&mut self, count: usize) -> Option<Vec<StreamRequestId>> {
        if count == 0 {
            return Some(Vec::new());
        }
        let first = self.next_request_id?;
        let count = u64::try_from(count).ok()?;
        let last = first.get().checked_add(count.checked_sub(1)?)?;

        let mut request_ids = Vec::with_capacity(usize::try_from(count).ok()?);
        for raw in first.get()..=last {
            request_ids.push(StreamRequestId::try_new(raw)?);
        }

        self.next_request_id = last.checked_add(1).and_then(StreamRequestId::try_new);
        Some(request_ids)
    }

    fn issue_candidates(
        &mut self,
        kind: StreamRequestKind,
        candidates: &[ChunkId],
        request_ids: &[StreamRequestId],
        requests: &mut Vec<StreamRequest>,
        events: &mut Vec<WorldStreamingEvent>,
    ) {
        debug_assert_eq!(candidates.len(), request_ids.len());
        for (&chunk_id, &request_id) in candidates.iter().zip(request_ids) {
            let record = self
                .records
                .get_mut(&chunk_id)
                .expect("queued candidate must reference a live record");
            let request = StreamRequest {
                request_id,
                chunk_id,
                kind,
                rank: record.rank,
            };
            record.operation = match kind {
                StreamRequestKind::Load => ChunkOperation::LoadRequested(request),
                StreamRequestKind::Unload => ChunkOperation::UnloadRequested(request),
            };
            let previous = self.pending_requests.insert(request_id, chunk_id);
            debug_assert!(previous.is_none());
            requests.push(request);
            events.push(WorldStreamingEvent::with_request(
                chunk_id,
                request_id,
                match kind {
                    StreamRequestKind::Load => WorldStreamingEventKind::LoadRequested,
                    StreamRequestKind::Unload => WorldStreamingEventKind::UnloadRequested,
                },
            ));
        }
    }

    fn finish_request(&mut self, request_id: StreamRequestId, chunk_id: ChunkId) {
        let removed = self.pending_requests.remove(&request_id);
        debug_assert_eq!(removed, Some(chunk_id));
    }

    fn prune_neutral_records(&mut self) {
        self.records.retain(|_, record| {
            record.desired
                || record.availability != ChunkAvailability::Absent
                || record.operation != ChunkOperation::Idle
                || record.blocking_failure.is_some()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ChunkOperation, ChunkRuntimeRecord, StreamingBudgets, StreamingTick, WorldStreamingConfig,
        WorldStreamingController,
    };
    use crate::StreamRequestId;
    use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
    use runen_spatial_demand::DemandLimits;

    fn controller() -> WorldStreamingController {
        let partition = GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap();
        let mut config = WorldStreamingConfig::new(WorldId(7), partition, DemandLimits::default());
        config.budgets = StreamingBudgets {
            max_load_requests_per_tick: 2,
            max_unload_requests_per_tick: 0,
        };
        WorldStreamingController::new(config)
    }

    #[test]
    fn request_id_reservation_never_reuses_the_terminal_identity() {
        let mut controller = controller();
        controller.next_request_id = StreamRequestId::try_new(u64::MAX);

        let terminal = controller.reserve_request_ids(1).unwrap();
        assert_eq!(terminal[0].get(), u64::MAX);
        assert!(controller.next_request_id.is_none());
        assert!(controller.reserve_request_ids(1).is_none());
    }

    #[test]
    fn exhausted_batch_issues_nothing_and_preserves_queued_operations() {
        let mut controller = controller();
        for x in [0, 1] {
            let chunk_id = ChunkId::new(WorldId(7), ChunkCoord3 { x, y: 0, z: 0 });
            let mut record = ChunkRuntimeRecord::new(chunk_id);
            record.desired = true;
            record.operation = ChunkOperation::LoadQueued;
            controller.records.insert(chunk_id, record);
        }
        controller.next_request_id = StreamRequestId::try_new(u64::MAX);

        let output = controller
            .tick(StreamingTick::without_demand_changes())
            .unwrap();

        assert!(output.request_id_exhausted);
        assert!(output.requests.is_empty());
        assert!(controller.pending_requests.is_empty());
        assert_eq!(controller.next_request_id.unwrap().get(), u64::MAX);
        assert!(
            controller
                .records()
                .all(|record| record.operation() == ChunkOperation::LoadQueued)
        );
    }
}
