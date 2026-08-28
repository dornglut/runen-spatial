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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StreamingCapacity {
    max_tracked_records: usize,
    max_in_flight_loads: usize,
    max_in_flight_unloads: usize,
}

impl StreamingCapacity {
    pub const fn new(
        max_tracked_records: usize,
        max_in_flight_loads: usize,
        max_in_flight_unloads: usize,
    ) -> Self {
        Self {
            max_tracked_records,
            max_in_flight_loads,
            max_in_flight_unloads,
        }
    }

    pub const fn max_tracked_records(self) -> usize {
        self.max_tracked_records
    }

    pub const fn max_in_flight_loads(self) -> usize {
        self.max_in_flight_loads
    }

    pub const fn max_in_flight_unloads(self) -> usize {
        self.max_in_flight_unloads
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldStreamingConfig {
    pub world_id: WorldId,
    pub partition: GridPartitionConfig,
    pub demand_limits: DemandLimits,
    pub budgets: StreamingBudgets,
    pub capacity: StreamingCapacity,
}

impl WorldStreamingConfig {
    pub fn new(
        world_id: WorldId,
        partition: GridPartitionConfig,
        demand_limits: DemandLimits,
        capacity: StreamingCapacity,
    ) -> Self {
        Self {
            world_id,
            partition,
            demand_limits,
            budgets: StreamingBudgets::default(),
            capacity,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct StreamingPressureDiagnostics {
    tracked_records: usize,
    max_tracked_records: usize,
    deferred_loads: usize,
    in_flight_loads: usize,
    max_in_flight_loads: usize,
    in_flight_unloads: usize,
    max_in_flight_unloads: usize,
    remaining_unloads: usize,
}

impl StreamingPressureDiagnostics {
    pub const fn tracked_records(self) -> usize {
        self.tracked_records
    }

    pub const fn max_tracked_records(self) -> usize {
        self.max_tracked_records
    }

    pub const fn deferred_loads(self) -> usize {
        self.deferred_loads
    }

    pub const fn in_flight_loads(self) -> usize {
        self.in_flight_loads
    }

    pub const fn max_in_flight_loads(self) -> usize {
        self.max_in_flight_loads
    }

    pub const fn in_flight_unloads(self) -> usize {
        self.in_flight_unloads
    }

    pub const fn max_in_flight_unloads(self) -> usize {
        self.max_in_flight_unloads
    }

    pub const fn remaining_unloads(self) -> usize {
        self.remaining_unloads
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StreamingTickOutput {
    pub requests: Vec<StreamRequest>,
    pub request_id_exhausted: bool,
    pub pressure: StreamingPressureDiagnostics,
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
    fn new_load(chunk_id: ChunkId, rank: DemandRank) -> Self {
        Self {
            chunk_id,
            desired: true,
            rank,
            availability: ChunkAvailability::Absent,
            operation: ChunkOperation::Idle,
            blocking_failure: None,
        }
    }

    pub const fn chunk_id(&self) -> ChunkId {
        self.chunk_id
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
    capacity: StreamingCapacity,
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
            capacity: config.capacity,
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

    pub const fn capacity(&self) -> StreamingCapacity {
        self.capacity
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
        if !tick.demand_changes.is_empty() {
            let delta = self
                .planner
                .apply_changes(tick.demand_changes)
                .map_err(WorldStreamingError::SpatialDemand)?;
            self.apply_demand_delta(&delta);
        }
        self.prune_neutral_records();

        let (in_flight_loads, in_flight_unloads) = self.in_flight_counts();
        let remaining_load_capacity = self
            .capacity
            .max_in_flight_loads
            .checked_sub(in_flight_loads)
            .expect("in-flight load count must not exceed configured capacity");
        let remaining_unload_capacity = self
            .capacity
            .max_in_flight_unloads
            .checked_sub(in_flight_unloads)
            .expect("in-flight unload count must not exceed configured capacity");
        let load_limit = self
            .budgets
            .max_load_requests_per_tick
            .min(remaining_load_capacity);
        let unload_limit = self
            .budgets
            .max_unload_requests_per_tick
            .min(remaining_unload_capacity);

        let load_candidates = self.load_candidates(load_limit);
        let unload_candidates = self.unload_candidates(unload_limit);
        let request_count = load_candidates.len() + unload_candidates.len();
        let Some(request_ids) = self.reserve_request_ids(request_count) else {
            return Ok(StreamingTickOutput {
                requests: Vec::new(),
                request_id_exhausted: true,
                pressure: self.pressure_diagnostics(),
            });
        };

        let (load_request_ids, unload_request_ids) = request_ids.split_at(load_candidates.len());
        let mut requests = Vec::with_capacity(request_count);
        self.issue_load_candidates(&load_candidates, load_request_ids, &mut requests);
        self.issue_unload_candidates(&unload_candidates, unload_request_ids, &mut requests);

        Ok(StreamingTickOutput {
            requests,
            request_id_exhausted: false,
            pressure: self.pressure_diagnostics(),
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
                events.push(WorldStreamingEvent::new(
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
                events.push(WorldStreamingEvent::new(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderCompleted,
                ));
                events.push(WorldStreamingEvent::new(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::Resident,
                ));
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
                events.push(WorldStreamingEvent::new(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderFailed,
                ));
            }
            (
                ChunkOperation::UnloadRequested(request),
                ProviderEventKind::Started,
                ChunkAvailability::Resident,
            ) => {
                self.records.get_mut(&event.chunk_id).unwrap().operation =
                    ChunkOperation::Unloading(request);
                events.push(WorldStreamingEvent::new(
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
                events.push(WorldStreamingEvent::new(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderCompleted,
                ));
                events.push(WorldStreamingEvent::new(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::Unloaded,
                ));
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
                events.push(WorldStreamingEvent::new(
                    event.chunk_id,
                    event.request_id,
                    WorldStreamingEventKind::ProviderFailed,
                ));
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

        if let Some(record) = self.records.get_mut(&event.chunk_id) {
            Self::reconcile_blocking_failure(record);
        }
        self.prune_neutral_records();
        Ok(events)
    }

    pub fn retry_blocking_failure(&mut self, chunk_id: ChunkId) -> Result<(), WorldStreamingError> {
        let Some(record) = self.records.get_mut(&chunk_id) else {
            return Err(WorldStreamingError::UnknownChunk { chunk_id });
        };

        match (
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
            )
            | (
                false,
                ChunkAvailability::Resident,
                ChunkOperation::Idle,
                Some(StreamRequestKind::Unload),
            ) => {
                record.blocking_failure = None;
            }
            _ => {
                return Err(WorldStreamingError::InvalidBlockingFailureRetry {
                    chunk_id,
                    desired: record.desired,
                    availability: record.availability,
                    operation: record.operation,
                    blocking_failure: record.blocking_failure,
                });
            }
        }

        self.prune_neutral_records();
        Ok(())
    }

    fn apply_demand_delta(&mut self, delta: &SpatialDemandDelta) {
        for chunk in delta.entered() {
            self.mark_desired(chunk.chunk_id(), chunk.rank());
        }
        for chunk in delta.updated() {
            self.mark_desired(chunk.chunk_id(), chunk.rank());
        }
        for chunk in delta.exited() {
            self.mark_undesired(chunk.chunk_id(), chunk.rank());
        }
    }

    fn mark_desired(&mut self, chunk_id: ChunkId, rank: DemandRank) {
        if let Some(record) = self.records.get_mut(&chunk_id) {
            record.desired = true;
            record.rank = rank;
            Self::reconcile_blocking_failure(record);
        }
    }

    fn mark_undesired(&mut self, chunk_id: ChunkId, rank: DemandRank) {
        if let Some(record) = self.records.get_mut(&chunk_id) {
            record.desired = false;
            record.rank = rank;
            Self::reconcile_blocking_failure(record);
        }
    }

    fn reconcile_blocking_failure(record: &mut ChunkRuntimeRecord) {
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
        if record
            .blocking_failure
            .is_some_and(|failure| failure != required_kind)
        {
            record.blocking_failure = None;
        }
    }

    fn load_candidates(&self, limit: usize) -> Vec<(ChunkId, DemandRank)> {
        let available_record_slots = self
            .capacity
            .max_tracked_records
            .checked_sub(self.records.len())
            .expect("tracked record count must not exceed configured capacity");
        let limit = limit.min(available_record_slots);
        if limit == 0 {
            return Vec::new();
        }

        let mut candidates = Vec::with_capacity(limit);
        for demanded in self.planner.effective_snapshot().chunks() {
            if self.records.contains_key(&demanded.chunk_id()) {
                continue;
            }
            candidates.push((demanded.chunk_id(), demanded.rank()));
            if candidates.len() == limit {
                break;
            }
        }
        candidates
    }

    fn unload_candidates(&self, limit: usize) -> Vec<(ChunkId, DemandRank)> {
        let mut candidates = self
            .records
            .values()
            .filter(|record| Self::record_is_unload_eligible(record))
            .map(|record| (record.chunk_id, record.rank))
            .collect::<Vec<_>>();
        candidates.sort_by_key(|(chunk_id, rank)| (*rank, *chunk_id));
        candidates.truncate(limit);
        candidates
    }

    fn record_is_unload_eligible(record: &ChunkRuntimeRecord) -> bool {
        !record.desired
            && record.availability == ChunkAvailability::Resident
            && record.operation == ChunkOperation::Idle
            && record.blocking_failure.is_none()
    }

    fn in_flight_counts(&self) -> (usize, usize) {
        self.records.values().fold((0, 0), |mut counts, record| {
            if let Some(request) = record.operation.active_request() {
                match request.kind {
                    StreamRequestKind::Load => counts.0 += 1,
                    StreamRequestKind::Unload => counts.1 += 1,
                }
            }
            counts
        })
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

    fn issue_load_candidates(
        &mut self,
        candidates: &[(ChunkId, DemandRank)],
        request_ids: &[StreamRequestId],
        requests: &mut Vec<StreamRequest>,
    ) {
        debug_assert_eq!(candidates.len(), request_ids.len());
        for (&(chunk_id, rank), &request_id) in candidates.iter().zip(request_ids) {
            debug_assert!(!self.records.contains_key(&chunk_id));
            debug_assert!(self.records.len() < self.capacity.max_tracked_records);
            self.records
                .insert(chunk_id, ChunkRuntimeRecord::new_load(chunk_id, rank));

            let record = self
                .records
                .get_mut(&chunk_id)
                .expect("selected load candidate must have a runtime record");
            let request = StreamRequest {
                request_id,
                chunk_id,
                kind: StreamRequestKind::Load,
                rank,
            };
            record.operation = ChunkOperation::LoadRequested(request);
            let previous = self.pending_requests.insert(request_id, chunk_id);
            debug_assert!(previous.is_none());
            requests.push(request);
        }
    }

    fn issue_unload_candidates(
        &mut self,
        candidates: &[(ChunkId, DemandRank)],
        request_ids: &[StreamRequestId],
        requests: &mut Vec<StreamRequest>,
    ) {
        debug_assert_eq!(candidates.len(), request_ids.len());
        for (&(chunk_id, rank), &request_id) in candidates.iter().zip(request_ids) {
            let record = self
                .records
                .get_mut(&chunk_id)
                .expect("selected unload candidate must have a runtime record");
            debug_assert!(Self::record_is_unload_eligible(record));
            let request = StreamRequest {
                request_id,
                chunk_id,
                kind: StreamRequestKind::Unload,
                rank,
            };
            record.operation = ChunkOperation::UnloadRequested(request);
            let previous = self.pending_requests.insert(request_id, chunk_id);
            debug_assert!(previous.is_none());
            requests.push(request);
        }
    }

    fn finish_request(&mut self, request_id: StreamRequestId, chunk_id: ChunkId) {
        let removed = self.pending_requests.remove(&request_id);
        debug_assert_eq!(removed, Some(chunk_id));
    }

    fn pressure_diagnostics(&self) -> StreamingPressureDiagnostics {
        let (in_flight_loads, in_flight_unloads) = self.in_flight_counts();
        let deferred_loads = self
            .planner
            .effective_snapshot()
            .chunks()
            .iter()
            .filter(|chunk| !self.records.contains_key(&chunk.chunk_id()))
            .count();
        let remaining_unloads = self
            .records
            .values()
            .filter(|record| Self::record_is_unload_eligible(record))
            .count();

        StreamingPressureDiagnostics {
            tracked_records: self.records.len(),
            max_tracked_records: self.capacity.max_tracked_records,
            deferred_loads,
            in_flight_loads,
            max_in_flight_loads: self.capacity.max_in_flight_loads,
            in_flight_unloads,
            max_in_flight_unloads: self.capacity.max_in_flight_unloads,
            remaining_unloads,
        }
    }

    fn prune_neutral_records(&mut self) {
        self.records.retain(|_, record| {
            record.availability != ChunkAvailability::Absent
                || record.operation != ChunkOperation::Idle
                || record.blocking_failure.is_some()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{StreamingCapacity, StreamingTick, WorldStreamingConfig, WorldStreamingController};
    use crate::StreamRequestId;
    use runen_spatial::{GridPartitionConfig, WorldId, WorldPosition};
    use runen_spatial_demand::{
        DemandFocus, DemandLimits, DemandSourceChange, DemandSourceId, DemandSourceSnapshot,
    };

    fn controller() -> WorldStreamingController {
        let partition = GridPartitionConfig::try_new(16.0, [8, 8, 8]).unwrap();
        let capacity = StreamingCapacity::new(8, 2, 2);
        let config =
            WorldStreamingConfig::new(WorldId(7), partition, DemandLimits::default(), capacity);
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
    fn request_id_batch_reservation_is_atomic_at_exhaustion() {
        let mut controller = controller();
        controller.next_request_id = StreamRequestId::try_new(u64::MAX);

        assert!(controller.reserve_request_ids(2).is_none());
        assert_eq!(controller.next_request_id.unwrap().get(), u64::MAX);
    }

    #[test]
    fn exhausted_tick_materializes_no_selected_load_records() {
        let mut controller = controller();
        controller.next_request_id = StreamRequestId::try_new(u64::MAX);
        let focus = DemandFocus::try_new(
            WorldPosition::try_new(WorldId(7), [0.0, 0.0, 0.0]).unwrap(),
            1,
            1,
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

        assert!(output.request_id_exhausted);
        assert!(output.requests.is_empty());
        assert_eq!(controller.records().count(), 0);
        assert_eq!(controller.pending_requests().count(), 0);
        assert_eq!(controller.effective_demand().len(), 9);
        assert_eq!(output.pressure.deferred_loads(), 9);
        assert_eq!(controller.next_request_id.unwrap().get(), u64::MAX);
    }
}
