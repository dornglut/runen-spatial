use crate::{ChunkLifecycleState, ProviderEventKind, StreamRequestId, StreamRequestKind};
use runen_spatial::{ChunkId, SpatialMathError};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WorldStreamingError {
    SpatialMath(SpatialMathError),
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
