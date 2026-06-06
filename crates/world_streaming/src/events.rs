use crate::StreamRequestId;
use spatial::ChunkId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProviderEventKind {
    Started,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ProviderEvent {
    pub request_id: StreamRequestId,
    pub chunk_id: ChunkId,
    pub kind: ProviderEventKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WorldStreamingEventKind {
    LoadQueued,
    LoadRequestCancelled,
    LoadRequested,
    ProviderStarted,
    ProviderCompleted,
    ProviderFailed,
    Resident,
    UnloadQueued,
    UnloadRequestCancelled,
    UnloadRequested,
    Unloaded,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WorldStreamingEvent {
    pub chunk_id: ChunkId,
    pub kind: WorldStreamingEventKind,
}

impl WorldStreamingEvent {
    pub fn new(chunk_id: ChunkId, kind: WorldStreamingEventKind) -> Self {
        Self { chunk_id, kind }
    }
}
