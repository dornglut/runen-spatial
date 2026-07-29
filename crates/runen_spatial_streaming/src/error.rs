use crate::{ChunkLifecycleState, ProviderEventKind, StreamRequestId, StreamRequestKind};
use runen_spatial::{ChunkId, SpatialMathError};
use runen_spatial_demand::SpatialDemandError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldStreamingError {
    SpatialMath(SpatialMathError),
    SpatialDemand(SpatialDemandError),
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
