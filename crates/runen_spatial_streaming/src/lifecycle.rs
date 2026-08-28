use crate::request::{StreamRequest, StreamRequestKind};

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
    pub const fn active_request(&self) -> Option<&StreamRequest> {
        match self {
            Self::LoadRequested(request)
            | Self::Loading(request)
            | Self::UnloadRequested(request)
            | Self::Unloading(request) => Some(request),
            Self::Idle | Self::LoadQueued | Self::UnloadQueued => None,
        }
    }

    pub const fn kind(self) -> Option<StreamRequestKind> {
        match self {
            Self::LoadQueued | Self::LoadRequested(_) | Self::Loading(_) => {
                Some(StreamRequestKind::Load)
            }
            Self::UnloadQueued | Self::UnloadRequested(_) | Self::Unloading(_) => {
                Some(StreamRequestKind::Unload)
            }
            Self::Idle => None,
        }
    }
}
