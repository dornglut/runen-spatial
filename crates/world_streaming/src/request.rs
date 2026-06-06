use crate::ChunkPriority;
use spatial::ChunkId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct StreamRequestId(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StreamRequestKind {
    Load,
    Unload,
    CancelLoad,
    CancelUnload,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StreamRequest {
    pub request_id: StreamRequestId,
    pub chunk_id: ChunkId,
    pub kind: StreamRequestKind,
    pub priority: ChunkPriority,
}
