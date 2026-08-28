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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DemandRank(u32);

impl DemandRank {
    pub(crate) fn from_bounded_index(index: usize) -> Self {
        Self(
            u32::try_from(index)
                .expect("effective demand index is bounded by the u32 effective-chunk limit"),
        )
    }

    pub const fn get(self) -> u32 {
        self.0
    }
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
