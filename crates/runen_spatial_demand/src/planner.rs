use crate::{
    DemandClass, DemandFocus, DemandLimits, DemandPressureDiagnostics, DemandRank,
    DemandSourceChange, DemandSourceId, DemandSourceSnapshot, DemandedChunk,
    EffectiveDemandSnapshot, SpatialDemandDelta, SpatialDemandError,
};
use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, SpatialMathError, WorldId};
use std::cmp::Ordering;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
struct SourceState {
    snapshot: DemandSourceSnapshot,
    focus_chunks: BTreeMap<ChunkCoord3, DemandClass>,
}

#[derive(Debug, Copy, Clone)]
struct Contribution {
    chunk_id: ChunkId,
    class: DemandClass,
    source_id: DemandSourceId,
    ordinal: u32,
}

#[derive(Debug, Clone)]
struct Candidate {
    sources: BTreeMap<DemandSourceId, SourceState>,
    snapshot: EffectiveDemandSnapshot,
    delta: SpatialDemandDelta,
}

pub struct SpatialDemandPlanner {
    world_id: WorldId,
    partition: GridPartitionConfig,
    limits: DemandLimits,
    sources: BTreeMap<DemandSourceId, SourceState>,
    snapshot: EffectiveDemandSnapshot,
}

impl SpatialDemandPlanner {
    pub fn new(world_id: WorldId, partition: GridPartitionConfig, limits: DemandLimits) -> Self {
        let pressure = DemandPressureDiagnostics::new(0, 0, 0, 0, 0);
        Self {
            world_id,
            partition,
            limits,
            sources: BTreeMap::new(),
            snapshot: EffectiveDemandSnapshot::new(Vec::new(), pressure),
        }
    }

    pub const fn world_id(&self) -> WorldId {
        self.world_id
    }

    pub fn partition(&self) -> &GridPartitionConfig {
        &self.partition
    }

    pub const fn limits(&self) -> DemandLimits {
        self.limits
    }

    pub fn effective_snapshot(&self) -> &EffectiveDemandSnapshot {
        &self.snapshot
    }

    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    pub fn replace_source(
        &mut self,
        source_id: DemandSourceId,
        snapshot: DemandSourceSnapshot,
    ) -> Result<SpatialDemandDelta, SpatialDemandError> {
        self.apply_changes([DemandSourceChange::Replace {
            source_id,
            snapshot,
        }])
    }

    pub fn remove_source(
        &mut self,
        source_id: DemandSourceId,
    ) -> Result<SpatialDemandDelta, SpatialDemandError> {
        self.apply_changes([DemandSourceChange::Remove { source_id }])
    }

    pub fn apply_changes(
        &mut self,
        changes: impl IntoIterator<Item = DemandSourceChange>,
    ) -> Result<SpatialDemandDelta, SpatialDemandError> {
        let changes = canonicalize_changes(changes)?;
        let candidate = self.changes_candidate(&changes)?;
        self.sources = candidate.sources;
        self.snapshot = candidate.snapshot;
        Ok(candidate.delta)
    }

    fn changes_candidate(
        &self,
        changes: &[DemandSourceChange],
    ) -> Result<Candidate, SpatialDemandError> {
        let mut snapshots = self
            .sources
            .iter()
            .map(|(source_id, state)| (*source_id, state.snapshot.clone()))
            .collect::<BTreeMap<_, _>>();
        for change in changes {
            match change {
                DemandSourceChange::Replace {
                    source_id,
                    snapshot,
                } => {
                    snapshots.insert(*source_id, snapshot.clone());
                }
                DemandSourceChange::Remove { source_id } => {
                    snapshots.remove(source_id);
                }
            }
        }

        self.validate_source_count(snapshots.len())?;

        let total_limit = usize::try_from(self.limits.max_total_contributions()).map_err(|_| {
            SpatialDemandError::CountOverflow {
                operation: "total contribution limit conversion",
            }
        })?;
        let mut total_contributions = 0_usize;
        for (source_id, snapshot) in &snapshots {
            let contribution_count = self.prospective_source_contribution_count(
                *source_id,
                snapshot,
                self.sources.get(source_id),
            )?;
            total_contributions = total_contributions.checked_add(contribution_count).ok_or(
                SpatialDemandError::CountOverflow {
                    operation: "total source contributions",
                },
            )?;
            if total_contributions > total_limit {
                return Err(SpatialDemandError::TotalContributionLimitExceeded {
                    limit: self.limits.max_total_contributions(),
                    candidate: total_contributions,
                });
            }
        }

        let mut sources = BTreeMap::new();
        for (source_id, snapshot) in snapshots {
            let existing = self.sources.get(&source_id);
            let state = if existing.is_some_and(|state| state.snapshot == snapshot) {
                existing.expect("existing source state was checked").clone()
            } else {
                self.replacement_state(source_id, snapshot, existing)?
            };
            sources.insert(source_id, state);
        }
        self.candidate(sources, total_contributions)
    }

    fn validate_source_count(&self, source_count: usize) -> Result<(), SpatialDemandError> {
        let limit = usize::try_from(self.limits.max_sources()).map_err(|_| {
            SpatialDemandError::CountOverflow {
                operation: "source limit conversion",
            }
        })?;
        if source_count > limit {
            return Err(SpatialDemandError::SourceLimitExceeded {
                limit: self.limits.max_sources(),
                candidate: source_count,
            });
        }
        Ok(())
    }

    fn validate_snapshot_world(
        &self,
        snapshot: &DemandSourceSnapshot,
    ) -> Result<(), SpatialDemandError> {
        if let Some(focus) = snapshot.focus()
            && focus.position().world_id() != self.world_id
        {
            return Err(SpatialMathError::WorldMismatch {
                expected: self.world_id,
                actual: focus.position().world_id(),
            }
            .into());
        }
        for pin in snapshot.pins() {
            if pin.world_id != self.world_id {
                return Err(SpatialMathError::WorldMismatch {
                    expected: self.world_id,
                    actual: pin.world_id,
                }
                .into());
            }
        }
        Ok(())
    }

    fn prospective_source_contribution_count(
        &self,
        source_id: DemandSourceId,
        snapshot: &DemandSourceSnapshot,
        previous: Option<&SourceState>,
    ) -> Result<usize, SpatialDemandError> {
        self.validate_snapshot_world(snapshot)?;

        if let Some(previous) = previous
            && previous.snapshot == *snapshot
        {
            return source_contribution_count(source_id, previous, self.limits);
        }

        let mut candidate;
        if let Some(focus) = snapshot.focus() {
            let center = self
                .partition
                .chunk_coord_from_world_position(focus.position())?;
            candidate = check_desired_volume(focus, source_id, self.limits)?;

            if let Some(previous) = previous {
                for chunk in previous.focus_chunks.keys().copied() {
                    let is_desired = inside_box(
                        chunk,
                        center,
                        focus.horizontal_desired_radius(),
                        focus.vertical_desired_radius(),
                    );
                    let is_retained = inside_box(
                        chunk,
                        center,
                        focus.horizontal_retain_radius(),
                        focus.vertical_retain_radius(),
                    );
                    if !is_desired && is_retained {
                        candidate =
                            candidate
                                .checked_add(1)
                                .ok_or(SpatialDemandError::CountOverflow {
                                    operation: "prospective source contributions",
                                })?;
                    }
                }
            }

            for pin in snapshot.pins() {
                let is_desired = inside_box(
                    pin.coord,
                    center,
                    focus.horizontal_desired_radius(),
                    focus.vertical_desired_radius(),
                );
                let overlaps_retained = previous.is_some_and(|previous| {
                    previous.focus_chunks.contains_key(&pin.coord)
                        && !is_desired
                        && inside_box(
                            pin.coord,
                            center,
                            focus.horizontal_retain_radius(),
                            focus.vertical_retain_radius(),
                        )
                });
                if !is_desired && !overlaps_retained {
                    candidate =
                        candidate
                            .checked_add(1)
                            .ok_or(SpatialDemandError::CountOverflow {
                                operation: "prospective source contributions",
                            })?;
                }
            }
        } else {
            candidate = snapshot.pins().count();
        }

        let limit = usize::try_from(self.limits.max_contributions_per_source()).map_err(|_| {
            SpatialDemandError::CountOverflow {
                operation: "per-source limit conversion",
            }
        })?;
        if candidate > limit {
            return Err(SpatialDemandError::PerSourceContributionLimitExceeded {
                source_id,
                limit: self.limits.max_contributions_per_source(),
                candidate,
            });
        }
        Ok(candidate)
    }

    fn replacement_state(
        &self,
        source_id: DemandSourceId,
        snapshot: DemandSourceSnapshot,
        previous: Option<&SourceState>,
    ) -> Result<SourceState, SpatialDemandError> {
        self.validate_snapshot_world(&snapshot)?;
        let mut focus_chunks = BTreeMap::new();
        if let Some(focus) = snapshot.focus() {
            let center = self
                .partition
                .chunk_coord_from_world_position(focus.position())?;
            let desired_volume = check_desired_volume(focus, source_id, self.limits)?;
            for chunk in chunk_box(
                center,
                focus.horizontal_desired_radius(),
                focus.vertical_desired_radius(),
                desired_volume,
            )? {
                insert_limited(
                    &mut focus_chunks,
                    chunk,
                    DemandClass::Desired,
                    source_id,
                    self.limits,
                )?;
            }
            if let Some(previous) = previous {
                for chunk in previous.focus_chunks.keys().copied() {
                    if inside_box(
                        chunk,
                        center,
                        focus.horizontal_retain_radius(),
                        focus.vertical_retain_radius(),
                    ) && !focus_chunks.contains_key(&chunk)
                    {
                        insert_limited(
                            &mut focus_chunks,
                            chunk,
                            DemandClass::Retained,
                            source_id,
                            self.limits,
                        )?;
                    }
                }
            }
        }
        Ok(SourceState {
            snapshot,
            focus_chunks,
        })
    }

    fn candidate(
        &self,
        sources: BTreeMap<DemandSourceId, SourceState>,
        total_contributions: usize,
    ) -> Result<Candidate, SpatialDemandError> {
        self.validate_source_count(sources.len())?;
        let mut effective = BTreeMap::<ChunkId, Contribution>::new();
        for (source_id, state) in &sources {
            for contribution in self.source_contributions(*source_id, state)? {
                match effective.get(&contribution.chunk_id) {
                    Some(previous)
                        if compare_contribution(previous, &contribution) != Ordering::Greater => {}
                    _ => {
                        effective.insert(contribution.chunk_id, contribution);
                    }
                }
            }
        }

        let mut candidates = effective.into_values().collect::<Vec<_>>();
        candidates.sort_by(compare_contribution);
        let candidate_count = candidates.len();
        let pinned = candidates
            .iter()
            .filter(|candidate| candidate.class == DemandClass::Pinned)
            .count();
        let capacity = usize::try_from(self.limits.max_effective_chunks()).map_err(|_| {
            SpatialDemandError::CountOverflow {
                operation: "effective limit conversion",
            }
        })?;
        if pinned > capacity {
            return Err(SpatialDemandError::PinnedCapacityExceeded {
                limit: self.limits.max_effective_chunks(),
                pinned,
            });
        }

        let demanded = candidates
            .into_iter()
            .take(capacity)
            .enumerate()
            .map(|(index, contribution)| {
                DemandedChunk::new(
                    contribution.chunk_id,
                    DemandRank::from_bounded_index(index),
                    contribution.class,
                )
            })
            .collect::<Vec<_>>();
        let pressure = DemandPressureDiagnostics::new(
            candidate_count,
            demanded.len(),
            pinned,
            total_contributions,
            sources.len(),
        );
        let snapshot = EffectiveDemandSnapshot::new(demanded, pressure);
        let delta = delta_between(&self.snapshot, &snapshot);
        Ok(Candidate {
            sources,
            snapshot,
            delta,
        })
    }

    fn source_contributions(
        &self,
        source_id: DemandSourceId,
        state: &SourceState,
    ) -> Result<Vec<Contribution>, SpatialDemandError> {
        let mut merged = state.focus_chunks.clone();
        for pin in state.snapshot.pins() {
            merged.insert(pin.coord, DemandClass::Pinned);
        }
        let limit = usize::try_from(self.limits.max_contributions_per_source()).map_err(|_| {
            SpatialDemandError::CountOverflow {
                operation: "per-source limit conversion",
            }
        })?;
        if merged.len() > limit {
            return Err(SpatialDemandError::PerSourceContributionLimitExceeded {
                source_id,
                limit: self.limits.max_contributions_per_source(),
                candidate: merged.len(),
            });
        }

        let center = state
            .snapshot
            .focus()
            .map(|focus| {
                self.partition
                    .chunk_coord_from_world_position(focus.position())
            })
            .transpose()?;
        let mut output = Vec::with_capacity(merged.len());
        for class in [
            DemandClass::Pinned,
            DemandClass::Desired,
            DemandClass::Retained,
        ] {
            let mut chunks = merged
                .iter()
                .filter_map(|(chunk, candidate_class)| {
                    (*candidate_class == class).then_some(*chunk)
                })
                .collect::<Vec<_>>();
            if class == DemandClass::Pinned {
                chunks.sort();
            } else if let Some(center) = center {
                chunks.sort_by_key(|chunk| (local_squared_distance(*chunk, center), *chunk));
            }
            for (index, chunk) in chunks.into_iter().enumerate() {
                output.push(Contribution {
                    chunk_id: ChunkId::new(self.world_id, chunk),
                    class,
                    source_id,
                    ordinal: u32::try_from(index).map_err(|_| {
                        SpatialDemandError::CountOverflow {
                            operation: "source-local ordinal",
                        }
                    })?,
                });
            }
        }
        Ok(output)
    }
}

fn canonicalize_changes(
    changes: impl IntoIterator<Item = DemandSourceChange>,
) -> Result<Vec<DemandSourceChange>, SpatialDemandError> {
    let mut changes = changes.into_iter().collect::<Vec<_>>();
    changes.sort_by_key(DemandSourceChange::source_id);
    for adjacent in changes.windows(2) {
        if adjacent[0].source_id() == adjacent[1].source_id() {
            return Err(SpatialDemandError::DuplicateSourceChange {
                source_id: adjacent[0].source_id(),
            });
        }
    }
    Ok(changes)
}

fn source_contribution_count(
    source_id: DemandSourceId,
    state: &SourceState,
    limits: DemandLimits,
) -> Result<usize, SpatialDemandError> {
    let additional_pins = state
        .snapshot
        .pins()
        .filter(|pin| !state.focus_chunks.contains_key(&pin.coord))
        .count();
    let candidate = state
        .focus_chunks
        .len()
        .checked_add(additional_pins)
        .ok_or(SpatialDemandError::CountOverflow {
            operation: "source contribution count",
        })?;
    let limit = usize::try_from(limits.max_contributions_per_source()).map_err(|_| {
        SpatialDemandError::CountOverflow {
            operation: "per-source limit conversion",
        }
    })?;
    if candidate > limit {
        return Err(SpatialDemandError::PerSourceContributionLimitExceeded {
            source_id,
            limit: limits.max_contributions_per_source(),
            candidate,
        });
    }
    Ok(candidate)
}

fn insert_limited(
    chunks: &mut BTreeMap<ChunkCoord3, DemandClass>,
    chunk: ChunkCoord3,
    class: DemandClass,
    source_id: DemandSourceId,
    limits: DemandLimits,
) -> Result<(), SpatialDemandError> {
    match chunks.get(&chunk) {
        Some(previous) if *previous <= class => {}
        _ => {
            chunks.insert(chunk, class);
        }
    }
    let limit = usize::try_from(limits.max_contributions_per_source()).map_err(|_| {
        SpatialDemandError::CountOverflow {
            operation: "per-source limit conversion",
        }
    })?;
    if chunks.len() > limit {
        return Err(SpatialDemandError::PerSourceContributionLimitExceeded {
            source_id,
            limit: limits.max_contributions_per_source(),
            candidate: chunks.len(),
        });
    }
    Ok(())
}

fn check_desired_volume(
    focus: DemandFocus,
    source_id: DemandSourceId,
    limits: DemandLimits,
) -> Result<usize, SpatialDemandError> {
    let horizontal = u128::from(focus.horizontal_desired_radius()) * 2 + 1;
    let vertical = u128::from(focus.vertical_desired_radius()) * 2 + 1;
    let volume = horizontal * vertical * horizontal;
    let candidate = usize::try_from(volume).map_err(|_| SpatialDemandError::CountOverflow {
        operation: "desired focus volume conversion",
    })?;
    let limit = usize::try_from(limits.max_contributions_per_source()).map_err(|_| {
        SpatialDemandError::CountOverflow {
            operation: "per-source limit conversion",
        }
    })?;
    if candidate > limit {
        return Err(SpatialDemandError::PerSourceContributionLimitExceeded {
            source_id,
            limit: limits.max_contributions_per_source(),
            candidate,
        });
    }
    Ok(candidate)
}

fn chunk_box(
    center: ChunkCoord3,
    horizontal_radius: u32,
    vertical_radius: u32,
    capacity: usize,
) -> Result<Vec<ChunkCoord3>, SpatialDemandError> {
    let horizontal = i64::from(horizontal_radius);
    let vertical = i64::from(vertical_radius);
    let min_x = center
        .x
        .checked_sub(horizontal)
        .ok_or(SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        })?;
    let max_x = center
        .x
        .checked_add(horizontal)
        .ok_or(SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        })?;
    let min_y = center
        .y
        .checked_sub(vertical)
        .ok_or(SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        })?;
    let max_y = center
        .y
        .checked_add(vertical)
        .ok_or(SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        })?;
    let min_z = center
        .z
        .checked_sub(horizontal)
        .ok_or(SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        })?;
    let max_z = center
        .z
        .checked_add(horizontal)
        .ok_or(SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        })?;
    let mut chunks = Vec::with_capacity(capacity);
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                chunks.push(ChunkCoord3 { x, y, z });
            }
        }
    }
    Ok(chunks)
}

fn inside_box(
    chunk: ChunkCoord3,
    center: ChunkCoord3,
    horizontal_radius: u32,
    vertical_radius: u32,
) -> bool {
    chunk.x.abs_diff(center.x) <= u64::from(horizontal_radius)
        && chunk.y.abs_diff(center.y) <= u64::from(vertical_radius)
        && chunk.z.abs_diff(center.z) <= u64::from(horizontal_radius)
}

fn local_squared_distance(a: ChunkCoord3, b: ChunkCoord3) -> u128 {
    let dx = u128::from(a.x.abs_diff(b.x));
    let dy = u128::from(a.y.abs_diff(b.y));
    let dz = u128::from(a.z.abs_diff(b.z));
    dx * dx + dy * dy + dz * dz
}

fn compare_contribution(left: &Contribution, right: &Contribution) -> Ordering {
    left.class
        .cmp(&right.class)
        .then_with(|| left.ordinal.cmp(&right.ordinal))
        .then_with(|| left.source_id.cmp(&right.source_id))
        .then_with(|| left.chunk_id.coord.cmp(&right.chunk_id.coord))
}

fn delta_between(
    previous: &EffectiveDemandSnapshot,
    next: &EffectiveDemandSnapshot,
) -> SpatialDemandDelta {
    let old = previous
        .chunks()
        .iter()
        .map(|chunk| (chunk.chunk_id(), *chunk))
        .collect::<BTreeMap<_, _>>();
    let new = next
        .chunks()
        .iter()
        .map(|chunk| (chunk.chunk_id(), *chunk))
        .collect::<BTreeMap<_, _>>();
    let mut entered = new
        .iter()
        .filter_map(|(id, chunk)| (!old.contains_key(id)).then_some(*chunk))
        .collect::<Vec<_>>();
    let mut updated = new
        .iter()
        .filter_map(|(id, chunk)| old.get(id).filter(|old| *old != chunk).map(|_| *chunk))
        .collect::<Vec<_>>();
    let mut exited = old
        .iter()
        .filter_map(|(id, chunk)| (!new.contains_key(id)).then_some(*chunk))
        .collect::<Vec<_>>();
    entered.sort_by_key(|chunk| chunk.rank());
    updated.sort_by_key(|chunk| chunk.rank());
    exited.sort_by_key(|chunk| chunk.rank());
    SpatialDemandDelta::new(entered, updated, exited, next.pressure())
}
