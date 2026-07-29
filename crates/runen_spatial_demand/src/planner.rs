use crate::{
    DemandClass, DemandDistanceOrder, DemandFocus, DemandLimits, DemandPressureDiagnostics,
    DemandRank, DemandSourceChange, DemandSourceId, DemandSourceSnapshot, DemandTransaction,
    DemandedChunk, EffectiveDemandSnapshot, SpatialDemandDelta, SpatialDemandError,
};
use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
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
    source_priority: u32,
    ordinal: u32,
    descending_coordinate: bool,
}

#[derive(Debug, Clone)]
struct Candidate {
    sources: BTreeMap<DemandSourceId, SourceState>,
    limits: DemandLimits,
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
        self.apply_transaction(DemandTransaction::try_new([DemandSourceChange::Replace {
            source_id,
            snapshot,
        }])?)
    }
    pub fn remove_source(
        &mut self,
        source_id: DemandSourceId,
    ) -> Result<SpatialDemandDelta, SpatialDemandError> {
        self.apply_transaction(DemandTransaction::try_new([DemandSourceChange::Remove {
            source_id,
        }])?)
    }
    pub fn apply_transaction(
        &mut self,
        transaction: DemandTransaction,
    ) -> Result<SpatialDemandDelta, SpatialDemandError> {
        self.apply_transaction_with(transaction, |_, delta| delta.clone())
    }
    pub fn apply_transaction_with<T>(
        &mut self,
        transaction: DemandTransaction,
        prepare: impl FnOnce(&EffectiveDemandSnapshot, &SpatialDemandDelta) -> T,
    ) -> Result<T, SpatialDemandError> {
        let candidate = self.transaction_candidate(transaction)?;
        let prepared = prepare(&candidate.snapshot, &candidate.delta);
        self.sources = candidate.sources;
        self.snapshot = candidate.snapshot;
        Ok(prepared)
    }
    pub fn replace_limits(
        &mut self,
        limits: DemandLimits,
    ) -> Result<SpatialDemandDelta, SpatialDemandError> {
        self.replace_limits_with(limits, |_, delta| delta.clone())
    }
    pub fn replace_limits_with<T>(
        &mut self,
        limits: DemandLimits,
        prepare: impl FnOnce(&EffectiveDemandSnapshot, &SpatialDemandDelta) -> T,
    ) -> Result<T, SpatialDemandError> {
        let candidate = self.candidate(self.sources.clone(), limits)?;
        let prepared = prepare(&candidate.snapshot, &candidate.delta);
        self.limits = candidate.limits;
        self.sources = candidate.sources;
        self.snapshot = candidate.snapshot;
        Ok(prepared)
    }

    fn transaction_candidate(
        &self,
        transaction: DemandTransaction,
    ) -> Result<Candidate, SpatialDemandError> {
        let mut snapshots = self
            .sources
            .iter()
            .map(|(source_id, state)| (*source_id, state.snapshot.clone()))
            .collect::<BTreeMap<_, _>>();
        for change in transaction.changes() {
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

        self.validate_source_count(snapshots.len(), self.limits)?;

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
        self.candidate(sources, self.limits)
    }

    fn validate_source_count(
        &self,
        source_count: usize,
        limits: DemandLimits,
    ) -> Result<(), SpatialDemandError> {
        let source_limit = usize::try_from(limits.max_sources()).map_err(|_| {
            SpatialDemandError::CountOverflow {
                operation: "source limit conversion",
            }
        })?;
        if source_count > source_limit {
            return Err(SpatialDemandError::SourceLimitExceeded {
                limit: limits.max_sources(),
                candidate: source_count,
            });
        }
        Ok(())
    }

    fn replacement_state(
        &self,
        source_id: DemandSourceId,
        snapshot: DemandSourceSnapshot,
        previous: Option<&SourceState>,
    ) -> Result<SourceState, SpatialDemandError> {
        let mut focus_chunks = BTreeMap::new();
        if let Some(focus) = snapshot.focus() {
            if focus.position().world_id() != self.world_id {
                return Err(SpatialDemandError::SpatialMath(
                    runen_spatial::SpatialMathError::WorldMismatch {
                        expected: self.world_id,
                        actual: focus.position().world_id(),
                    },
                ));
            }
            let center = self
                .partition
                .chunk_coord_from_world_position(focus.position())?;
            check_desired_volume(focus, source_id, self.limits)?;
            let desired = chunk_box(
                center,
                focus.horizontal_desired_radius(),
                focus.vertical_desired_radius(),
            )?;
            for chunk in desired {
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
                    )? && !focus_chunks.contains_key(&chunk)
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
        let mut all = focus_chunks.clone();
        for pin in snapshot.pins().copied() {
            insert_limited(&mut all, pin, DemandClass::Pinned, source_id, self.limits)?;
        }
        Ok(SourceState {
            snapshot,
            focus_chunks,
        })
    }

    fn candidate(
        &self,
        sources: BTreeMap<DemandSourceId, SourceState>,
        limits: DemandLimits,
    ) -> Result<Candidate, SpatialDemandError> {
        self.validate_source_count(sources.len(), limits)?;
        let mut total = 0_usize;
        let mut effective = BTreeMap::<ChunkId, Contribution>::new();
        for (source_id, state) in &sources {
            let contributions = self.source_contributions(*source_id, state, limits)?;
            total = total.checked_add(contributions.len()).ok_or(
                SpatialDemandError::CountOverflow {
                    operation: "total source contributions",
                },
            )?;
            if total
                > usize::try_from(limits.max_total_contributions()).map_err(|_| {
                    SpatialDemandError::CountOverflow {
                        operation: "total contribution limit conversion",
                    }
                })?
            {
                return Err(SpatialDemandError::TotalContributionLimitExceeded {
                    limit: limits.max_total_contributions(),
                    candidate: total,
                });
            }
            for contribution in contributions {
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
        let candidate_count = candidates.len();
        candidates.sort_by(compare_contribution);
        let pinned = candidates
            .iter()
            .filter(|candidate| candidate.class == DemandClass::Pinned)
            .count();
        let capacity = usize::try_from(limits.max_effective_chunks()).map_err(|_| {
            SpatialDemandError::CountOverflow {
                operation: "effective limit conversion",
            }
        })?;
        if pinned > capacity {
            return Err(SpatialDemandError::PinnedCapacityExceeded {
                limit: limits.max_effective_chunks(),
                pinned,
            });
        }
        let selected = candidates
            .into_iter()
            .enumerate()
            .filter_map(|(index, contribution)| {
                if contribution.class == DemandClass::Pinned || index < capacity {
                    Some(contribution)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let demanded = selected
            .into_iter()
            .enumerate()
            .map(|(index, contribution)| {
                Ok(DemandedChunk::new(
                    contribution.chunk_id,
                    DemandRank::try_from_index(index)?,
                    contribution.class,
                    contribution.source_id,
                ))
            })
            .collect::<Result<Vec<_>, SpatialDemandError>>()?;
        let pressure = DemandPressureDiagnostics::new(
            candidate_count,
            demanded.len(),
            pinned,
            total,
            sources.len(),
        );
        let snapshot = EffectiveDemandSnapshot::new(demanded, pressure);
        let delta = delta_between(&self.snapshot, &snapshot);
        Ok(Candidate {
            sources,
            limits,
            snapshot,
            delta,
        })
    }

    fn source_contributions(
        &self,
        source_id: DemandSourceId,
        state: &SourceState,
        limits: DemandLimits,
    ) -> Result<Vec<Contribution>, SpatialDemandError> {
        let mut merged = state.focus_chunks.clone();
        for pin in state.snapshot.pins().copied() {
            merged.insert(pin, DemandClass::Pinned);
        }
        let limit = usize::try_from(limits.max_contributions_per_source()).map_err(|_| {
            SpatialDemandError::CountOverflow {
                operation: "per-source limit conversion",
            }
        })?;
        if merged.len() > limit {
            return Err(SpatialDemandError::PerSourceContributionLimitExceeded {
                source_id,
                limit: limits.max_contributions_per_source(),
                candidate: merged.len(),
            });
        }
        let center = match state.snapshot.focus() {
            Some(focus) => Some(
                self.partition
                    .chunk_coord_from_world_position(focus.position())?,
            ),
            None => None,
        };
        let order = state
            .snapshot
            .focus()
            .map(|focus| focus.distance_order())
            .unwrap_or(DemandDistanceOrder::NearestFirst);
        let mut output = Vec::new();
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
            let descending =
                class != DemandClass::Pinned && matches!(order, DemandDistanceOrder::FarthestFirst);
            if class == DemandClass::Pinned {
                chunks.sort();
            } else if let Some(center) = center {
                let mut ranked = chunks
                    .into_iter()
                    .map(|chunk| Ok((checked_distance(chunk, center)?, chunk)))
                    .collect::<Result<Vec<_>, SpatialDemandError>>()?;
                ranked.sort_by(
                    |(left_distance, left), (right_distance, right)| match order {
                        DemandDistanceOrder::NearestFirst => left_distance
                            .cmp(right_distance)
                            .then_with(|| left.cmp(right)),
                        DemandDistanceOrder::FarthestFirst => right_distance
                            .cmp(left_distance)
                            .then_with(|| right.cmp(left)),
                    },
                );
                chunks = ranked.into_iter().map(|(_, chunk)| chunk).collect();
            }
            for (index, chunk) in chunks.into_iter().enumerate() {
                output.push(Contribution {
                    chunk_id: ChunkId::new(self.world_id, chunk),
                    class,
                    source_id,
                    source_priority: state.snapshot.priority().get(),
                    ordinal: u32::try_from(index).map_err(|_| {
                        SpatialDemandError::CountOverflow {
                            operation: "source-local ordinal",
                        }
                    })?,
                    descending_coordinate: descending,
                });
            }
        }
        Ok(output)
    }
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
) -> Result<(), SpatialDemandError> {
    let horizontal = u128::from(focus.horizontal_desired_radius())
        .checked_mul(2)
        .and_then(|value| value.checked_add(1))
        .ok_or(SpatialDemandError::CountOverflow {
            operation: "desired horizontal length",
        })?;
    let vertical = u128::from(focus.vertical_desired_radius())
        .checked_mul(2)
        .and_then(|value| value.checked_add(1))
        .ok_or(SpatialDemandError::CountOverflow {
            operation: "desired vertical length",
        })?;
    let volume = horizontal
        .checked_mul(vertical)
        .and_then(|value| value.checked_mul(horizontal))
        .ok_or(SpatialDemandError::CountOverflow {
            operation: "desired focus volume",
        })?;
    let candidate = usize::try_from(volume).map_err(|_| SpatialDemandError::CountOverflow {
        operation: "desired focus volume conversion",
    })?;
    if candidate
        > usize::try_from(limits.max_contributions_per_source()).map_err(|_| {
            SpatialDemandError::CountOverflow {
                operation: "per-source limit conversion",
            }
        })?
    {
        return Err(SpatialDemandError::PerSourceContributionLimitExceeded {
            source_id,
            limit: limits.max_contributions_per_source(),
            candidate,
        });
    }
    Ok(())
}

fn chunk_box(
    center: ChunkCoord3,
    horizontal_radius: u32,
    vertical_radius: u32,
) -> Result<Vec<ChunkCoord3>, SpatialDemandError> {
    let horizontal = i64::from(horizontal_radius);
    let vertical = i64::from(vertical_radius);
    let min_x = center.x.checked_sub(horizontal).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        },
    )?;
    let max_x = center.x.checked_add(horizontal).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        },
    )?;
    let min_y = center.y.checked_sub(vertical).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        },
    )?;
    let max_y = center.y.checked_add(vertical).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        },
    )?;
    let min_z = center.z.checked_sub(horizontal).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        },
    )?;
    let max_z = center.z.checked_add(horizontal).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand chunk range",
        },
    )?;
    let mut chunks = Vec::new();
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
) -> Result<bool, SpatialDemandError> {
    let horizontal = i64::from(horizontal_radius);
    let vertical = i64::from(vertical_radius);
    let min_x = center.x.checked_sub(horizontal).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand retain range",
        },
    )?;
    let max_x = center.x.checked_add(horizontal).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand retain range",
        },
    )?;
    let min_y = center.y.checked_sub(vertical).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand retain range",
        },
    )?;
    let max_y = center.y.checked_add(vertical).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand retain range",
        },
    )?;
    let min_z = center.z.checked_sub(horizontal).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand retain range",
        },
    )?;
    let max_z = center.z.checked_add(horizontal).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand retain range",
        },
    )?;
    Ok((min_x..=max_x).contains(&chunk.x)
        && (min_y..=max_y).contains(&chunk.y)
        && (min_z..=max_z).contains(&chunk.z))
}

fn checked_distance(a: ChunkCoord3, b: ChunkCoord3) -> Result<i128, SpatialDemandError> {
    let dx = i128::from(a.x).checked_sub(i128::from(b.x)).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand distance delta",
        },
    )?;
    let dy = i128::from(a.y).checked_sub(i128::from(b.y)).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand distance delta",
        },
    )?;
    let dz = i128::from(a.z).checked_sub(i128::from(b.z)).ok_or(
        runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand distance delta",
        },
    )?;
    let x = dx
        .checked_mul(dx)
        .ok_or(runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand distance square",
        })?;
    let y = dy
        .checked_mul(dy)
        .ok_or(runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand distance square",
        })?;
    let z = dz
        .checked_mul(dz)
        .ok_or(runen_spatial::SpatialMathError::ArithmeticOverflow {
            operation: "demand distance square",
        })?;
    x.checked_add(y)
        .and_then(|sum| sum.checked_add(z))
        .ok_or(SpatialDemandError::SpatialMath(
            runen_spatial::SpatialMathError::ArithmeticOverflow {
                operation: "demand distance sum",
            },
        ))
}

fn compare_contribution(left: &Contribution, right: &Contribution) -> Ordering {
    left.class
        .cmp(&right.class)
        .then_with(|| right.source_priority.cmp(&left.source_priority))
        .then_with(|| left.ordinal.cmp(&right.ordinal))
        .then_with(|| left.source_id.cmp(&right.source_id))
        .then_with(|| {
            if left.descending_coordinate {
                right.chunk_id.coord.cmp(&left.chunk_id.coord)
            } else {
                left.chunk_id.coord.cmp(&right.chunk_id.coord)
            }
        })
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
