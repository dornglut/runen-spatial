use crate::{DemandFocus, DemandSourceId, SpatialDemandError};
use runen_spatial::ChunkId;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub struct DemandSourceSnapshot {
    focus: Option<DemandFocus>,
    pins: BTreeSet<ChunkId>,
}

impl DemandSourceSnapshot {
    pub fn try_new(
        focus: Option<DemandFocus>,
        pins: impl IntoIterator<Item = ChunkId>,
    ) -> Result<Self, SpatialDemandError> {
        let pins = pins.into_iter().collect::<BTreeSet<_>>();
        if focus.is_none() && pins.is_empty() {
            return Err(SpatialDemandError::EmptySourceSnapshot);
        }
        Ok(Self { focus, pins })
    }

    pub const fn focus(&self) -> Option<DemandFocus> {
        self.focus
    }

    pub fn pins(&self) -> impl Iterator<Item = &ChunkId> {
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
