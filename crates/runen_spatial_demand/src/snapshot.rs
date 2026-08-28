use crate::{DemandClass, DemandRank};
use runen_spatial::ChunkId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DemandPressureDiagnostics {
    candidate_effective_chunks: usize,
    selected_effective_chunks: usize,
    unique_pinned_effective_chunks: usize,
    suppressed_effective_chunks: usize,
    total_source_contributions: usize,
    source_count: usize,
}

impl DemandPressureDiagnostics {
    pub(crate) const fn new(
        candidate_effective_chunks: usize,
        selected_effective_chunks: usize,
        unique_pinned_effective_chunks: usize,
        total_source_contributions: usize,
        source_count: usize,
    ) -> Self {
        Self {
            candidate_effective_chunks,
            selected_effective_chunks,
            unique_pinned_effective_chunks,
            suppressed_effective_chunks: candidate_effective_chunks - selected_effective_chunks,
            total_source_contributions,
            source_count,
        }
    }

    pub const fn candidate_effective_chunks(&self) -> usize {
        self.candidate_effective_chunks
    }

    pub const fn selected_effective_chunks(&self) -> usize {
        self.selected_effective_chunks
    }

    pub const fn unique_pinned_effective_chunks(&self) -> usize {
        self.unique_pinned_effective_chunks
    }

    pub const fn suppressed_effective_chunks(&self) -> usize {
        self.suppressed_effective_chunks
    }

    pub const fn total_source_contributions(&self) -> usize {
        self.total_source_contributions
    }

    pub const fn source_count(&self) -> usize {
        self.source_count
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DemandedChunk {
    chunk_id: ChunkId,
    rank: DemandRank,
    class: DemandClass,
}

impl DemandedChunk {
    pub(crate) const fn new(chunk_id: ChunkId, rank: DemandRank, class: DemandClass) -> Self {
        Self {
            chunk_id,
            rank,
            class,
        }
    }

    pub const fn chunk_id(&self) -> ChunkId {
        self.chunk_id
    }

    pub const fn rank(&self) -> DemandRank {
        self.rank
    }

    pub const fn class(&self) -> DemandClass {
        self.class
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveDemandSnapshot {
    chunks: Vec<DemandedChunk>,
    pressure: DemandPressureDiagnostics,
}

impl EffectiveDemandSnapshot {
    pub(crate) fn new(chunks: Vec<DemandedChunk>, pressure: DemandPressureDiagnostics) -> Self {
        Self { chunks, pressure }
    }

    pub fn chunks(&self) -> &[DemandedChunk] {
        &self.chunks
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn get(&self, chunk_id: ChunkId) -> Option<&DemandedChunk> {
        self.chunks.iter().find(|chunk| chunk.chunk_id == chunk_id)
    }

    pub const fn pressure(&self) -> DemandPressureDiagnostics {
        self.pressure
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialDemandDelta {
    entered: Vec<DemandedChunk>,
    updated: Vec<DemandedChunk>,
    exited: Vec<DemandedChunk>,
    pressure: DemandPressureDiagnostics,
}

impl SpatialDemandDelta {
    pub(crate) fn new(
        entered: Vec<DemandedChunk>,
        updated: Vec<DemandedChunk>,
        exited: Vec<DemandedChunk>,
        pressure: DemandPressureDiagnostics,
    ) -> Self {
        Self {
            entered,
            updated,
            exited,
            pressure,
        }
    }

    pub fn entered(&self) -> &[DemandedChunk] {
        &self.entered
    }

    pub fn updated(&self) -> &[DemandedChunk] {
        &self.updated
    }

    pub fn exited(&self) -> &[DemandedChunk] {
        &self.exited
    }

    pub const fn pressure(&self) -> DemandPressureDiagnostics {
        self.pressure
    }

    pub fn is_empty(&self) -> bool {
        self.entered.is_empty() && self.updated.is_empty() && self.exited.is_empty()
    }
}
