# Streaming Lifecycle

`world_streaming` is the payload-neutral request/event lifecycle controller.

The crate is designed from first principles. Runenwerk engine files such as
`engine/src/plugins/world/chunks/lifecycle.rs` are reference material only. They
must not be mechanically extracted because they contain engine, dirty-state,
build-generation, ECS-resource, and render-cache concepts.

## Provider Ownership

The crate uses request/event ownership:

```text
world_streaming emits StreamRequest.
Host or adapter performs loading.
Host reports ProviderEvent back.
world_streaming updates lifecycle.
```

Do not put provider traits, async runtimes, thread pools, filesystem IO, asset
catalogs, SDF stores, mesh generation, renderer uploads, or Godot scene logic in
the core.

## Lifecycle

Use this lifecycle:

```text
Absent
  -> LoadQueued
  -> LoadRequested
  -> Loading
  -> Resident
  -> UnloadQueued
  -> UnloadRequested
  -> Unloading
  -> Absent

Loading -> Failed
Resident -> Failed
Failed -> LoadQueued | Absent
```

`Queued` means the controller wants to spend budget on the transition.

`Requested` means the controller has emitted a request and is waiting for the
host/provider handoff.

`Loading` and `Unloading` mean the host has acknowledged provider work started.

`Resident` means the core knows the provider completed the chunk residency
transition. It does not imply visual, physics, gameplay, or save readiness.

## Public API Sketch

```rust
pub struct WorldStreamingController;
pub struct WorldStreamingConfig;
pub struct StreamingBudgets;
pub struct StreamingTick;
pub struct StreamingTickOutput;

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

pub enum StreamRequestKind {
    Load,
    Unload,
    CancelLoad,
    CancelUnload,
}

pub struct StreamRequest {
    pub request_id: StreamRequestId,
    pub chunk_id: spatial::ChunkId,
    pub kind: StreamRequestKind,
    pub priority: ChunkPriority,
}

pub enum ProviderEventKind {
    Started,
    Completed,
    Failed,
    Cancelled,
}

pub struct ProviderEvent {
    pub request_id: StreamRequestId,
    pub chunk_id: spatial::ChunkId,
    pub kind: ProviderEventKind,
}

pub enum WorldStreamingEventKind {
    LoadQueued,
    LoadRequested,
    ProviderStarted,
    ProviderCompleted,
    ProviderFailed,
    Resident,
    UnloadQueued,
    UnloadRequested,
    Unloaded,
}
```

Events must be deterministic and ordered by stable request priority, chunk id,
and request id.
