use crate::{ChunkAvailability, ChunkOperation, ProviderEventKind, StreamRequestId, StreamRequestKind};
use runen_spatial::ChunkId;
use runen_spatial_demand::SpatialDemandError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldStreamingError {
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
        event_kind: ProviderEventKind,
        availability: ChunkAvailability,
        operation: ChunkOperation,
    },
    InvalidBlockingFailureRetry {
        chunk_id: ChunkId,
        desired: bool,
        availability: ChunkAvailability,
        operation: ChunkOperation,
        blocking_failure: Option<StreamRequestKind>,
    },
}
