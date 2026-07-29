#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChunkLifecycleState {
    Absent,
    LoadQueued,
    LoadRequested,
    Loading,
    Resident,
    UnloadQueued,
    UnloadRequested,
    Unloading,
    Failed,
}
