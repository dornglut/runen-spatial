use crate::StreamRequestId;
use runen_spatial::ChunkId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProviderEventKind {
    Started,
    Completed,
    Failed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ProviderEvent {
    pub request_id: StreamRequestId,
    pub chunk_id: ChunkId,
    pub kind: ProviderEventKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WorldStreamingEventKind {
    LoadRequested,
    ProviderStarted,
    ProviderCompleted,
    ProviderFailed,
    Resident,
    UnloadRequested,
    Unloaded,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WorldStreamingEvent {
    pub chunk_id: ChunkId,
    pub request_id: Option<StreamRequestId>,
    pub kind: WorldStreamingEventKind,
}

impl WorldStreamingEvent {
    pub fn new(chunk_id: ChunkId, kind: WorldStreamingEventKind) -> Self {
        Self {
            chunk_id,
            request_id: None,
            kind,
        }
    }

    pub fn with_request(
        chunk_id: ChunkId,
        request_id: StreamRequestId,
        kind: WorldStreamingEventKind,
    ) -> Self {
        Self {
            chunk_id,
            request_id: Some(request_id),
            kind,
        }
    }
}
