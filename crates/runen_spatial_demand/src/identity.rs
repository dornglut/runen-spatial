#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DemandSourceId(u64);

impl DemandSourceId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DemandSourcePriority(u32);

impl DemandSourcePriority {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DemandRank(u32);

impl DemandRank {
    pub fn try_from_index(index: usize) -> Result<Self, crate::SpatialDemandError> {
        u32::try_from(index)
            .map(Self)
            .map_err(|_| crate::SpatialDemandError::EffectiveRankOverflow { candidate: index })
    }
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DemandDistanceOrder {
    NearestFirst,
    FarthestFirst,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DemandClass {
    Pinned,
    Desired,
    Retained,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DemandAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DemandLimitKind {
    Sources,
    ContributionsPerSource,
    TotalContributions,
    EffectiveChunks,
}
