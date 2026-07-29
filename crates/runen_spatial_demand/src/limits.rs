use crate::{DemandLimitKind, SpatialDemandError};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DemandLimits {
    max_sources: u32,
    max_contributions_per_source: u32,
    max_total_contributions: u32,
    max_effective_chunks: u32,
}

impl DemandLimits {
    pub fn try_new(
        max_sources: u32,
        max_contributions_per_source: u32,
        max_total_contributions: u32,
        max_effective_chunks: u32,
    ) -> Result<Self, SpatialDemandError> {
        for (value, limit) in [
            (max_sources, DemandLimitKind::Sources),
            (
                max_contributions_per_source,
                DemandLimitKind::ContributionsPerSource,
            ),
            (max_total_contributions, DemandLimitKind::TotalContributions),
            (max_effective_chunks, DemandLimitKind::EffectiveChunks),
        ] {
            if value == 0 {
                return Err(SpatialDemandError::ZeroLimit { limit });
            }
        }
        Ok(Self {
            max_sources,
            max_contributions_per_source,
            max_total_contributions,
            max_effective_chunks,
        })
    }
    pub const fn max_sources(&self) -> u32 {
        self.max_sources
    }
    pub const fn max_contributions_per_source(&self) -> u32 {
        self.max_contributions_per_source
    }
    pub const fn max_total_contributions(&self) -> u32 {
        self.max_total_contributions
    }
    pub const fn max_effective_chunks(&self) -> u32 {
        self.max_effective_chunks
    }
}

impl Default for DemandLimits {
    fn default() -> Self {
        Self::try_new(64, 16_384, 262_144, 65_536).expect("default demand limits are valid")
    }
}
