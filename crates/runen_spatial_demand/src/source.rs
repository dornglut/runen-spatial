use crate::{DemandFocus, DemandSourceId, DemandSourcePriority, SpatialDemandError};
use runen_spatial::ChunkCoord3;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub struct DemandSourceSnapshot {
    priority: DemandSourcePriority,
    focus: Option<DemandFocus>,
    pins: BTreeSet<ChunkCoord3>,
}

impl DemandSourceSnapshot {
    pub fn try_new(
        priority: DemandSourcePriority,
        focus: Option<DemandFocus>,
        pins: impl IntoIterator<Item = ChunkCoord3>,
    ) -> Result<Self, SpatialDemandError> {
        let pins = pins.into_iter().collect::<BTreeSet<_>>();
        if focus.is_none() && pins.is_empty() {
            return Err(SpatialDemandError::EmptySourceSnapshot);
        }
        Ok(Self {
            priority,
            focus,
            pins,
        })
    }
    pub const fn priority(&self) -> DemandSourcePriority {
        self.priority
    }
    pub const fn focus(&self) -> Option<DemandFocus> {
        self.focus
    }
    pub fn pins(&self) -> impl Iterator<Item = &ChunkCoord3> {
        self.pins.iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DemandSourceChange {
    Replace {
        source_id: DemandSourceId,
        snapshot: DemandSourceSnapshot,
    },
    Remove {
        source_id: DemandSourceId,
    },
}

impl DemandSourceChange {
    pub const fn source_id(&self) -> DemandSourceId {
        match self {
            Self::Replace { source_id, .. } | Self::Remove { source_id } => *source_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DemandTransaction {
    changes: Vec<DemandSourceChange>,
}

impl DemandTransaction {
    pub fn try_new(
        changes: impl IntoIterator<Item = DemandSourceChange>,
    ) -> Result<Self, SpatialDemandError> {
        let mut changes = changes.into_iter().collect::<Vec<_>>();
        changes.sort_by_key(DemandSourceChange::source_id);
        for adjacent in changes.windows(2) {
            if adjacent[0].source_id() == adjacent[1].source_id() {
                return Err(SpatialDemandError::DuplicateSourceChange {
                    source_id: adjacent[0].source_id(),
                });
            }
        }
        Ok(Self { changes })
    }
    pub fn changes(&self) -> impl Iterator<Item = &DemandSourceChange> {
        self.changes.iter()
    }
}
