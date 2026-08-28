use crate::{DemandAxis, DemandLimitKind, DemandSourceId};
use runen_spatial::SpatialMathError;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpatialDemandError {
    SpatialMath(SpatialMathError),
    EmptySourceSnapshot,
    RetainRadiusBelowDesired {
        axis: DemandAxis,
        desired: u32,
        retain: u32,
    },
    ZeroLimit {
        limit: DemandLimitKind,
    },
    DuplicateSourceChange {
        source_id: DemandSourceId,
    },
    SourceLimitExceeded {
        limit: u32,
        candidate: usize,
    },
    PerSourceContributionLimitExceeded {
        source_id: DemandSourceId,
        limit: u32,
        candidate: usize,
    },
    TotalContributionLimitExceeded {
        limit: u32,
        candidate: usize,
    },
    PinnedCapacityExceeded {
        limit: u32,
        pinned: usize,
    },
    CountOverflow {
        operation: &'static str,
    },
}

impl From<SpatialMathError> for SpatialDemandError {
    fn from(value: SpatialMathError) -> Self {
        Self::SpatialMath(value)
    }
}

impl fmt::Display for SpatialDemandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for SpatialDemandError {}
