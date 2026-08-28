use runen_spatial::ChunkId;
use runen_spatial_demand::DemandRank;
use std::num::NonZeroU64;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamRequestId(NonZeroU64);

impl StreamRequestId {
    pub const fn try_new(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StreamRequestKind {
    Load,
    Unload,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StreamRequest {
    pub request_id: StreamRequestId,
    pub chunk_id: ChunkId,
    pub kind: StreamRequestKind,
    pub rank: DemandRank,
}
