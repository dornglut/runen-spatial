use crate::request::StreamRequest;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChunkAvailability {
    Absent,
    Resident,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChunkOperation {
    Idle,
    LoadQueued,
    LoadRequested(StreamRequest),
    Loading(StreamRequest),
    UnloadQueued,
    UnloadRequested(StreamRequest),
    Unloading(StreamRequest),
}

impl ChunkOperation {
    pub(crate) const fn active_request(&self) -> Option<&StreamRequest> {
        match self {
            Self::LoadRequested(request)
            | Self::Loading(request)
            | Self::UnloadRequested(request)
            | Self::Unloading(request) => Some(request),
            Self::Idle | Self::LoadQueued | Self::UnloadQueued => None,
        }
    }
}
