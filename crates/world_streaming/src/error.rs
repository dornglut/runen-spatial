use crate::{ChunkLifecycleState, ProviderEventKind, StreamRequestId, StreamRequestKind};
use spatial::ChunkId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WorldStreamingError {
    UnknownChunk {
        chunk_id: ChunkId,
    },
    UnknownRequest {
        request_id: StreamRequestId,
    },
    RequestChunkMismatch {
        request_id: StreamRequestId,
        expected: ChunkId,
        actual: ChunkId,
    },
    InvalidProviderEvent {
        request_id: StreamRequestId,
        request_kind: StreamRequestKind,
        event_kind: ProviderEventKind,
        state: ChunkLifecycleState,
    },
    InvalidResidentFailure {
        chunk_id: ChunkId,
        state: ChunkLifecycleState,
    },
    InvalidFailedRetry {
        chunk_id: ChunkId,
        state: ChunkLifecycleState,
        desired: bool,
    },
}
