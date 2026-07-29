# Streaming Lifecycle

`runen-spatial-streaming` is the payload-neutral request/event lifecycle controller.

The crate is designed from first principles. Runenwerk engine files such as
`engine/src/plugins/world/chunks/lifecycle.rs` are reference material only. They
must not be mechanically extracted because they contain engine, dirty-state,
build-generation, ECS-resource, and render-cache concepts.

## Provider Ownership

The crate uses request/event ownership:

```text
runen-spatial-streaming emits StreamRequest.
Host or adapter performs loading.
Host reports ProviderEvent back.
runen-spatial-streaming updates lifecycle.
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
Failed -> LoadQueued by explicit retry
Failed -> Absent when no longer desired
```

`Queued` means the controller wants to spend budget on the transition.

`Requested` means the controller has emitted a request and is waiting for the
host/provider handoff.

`Loading` and `Unloading` mean the host has acknowledged provider work started.

`Resident` means the core knows the provider completed the chunk residency
transition. It does not imply visual, physics, gameplay, or save readiness.

Provider work is non-cancellable in the current contract. If desired state
changes while a load or unload request is active, the controller records the new
desired state and waits for the provider to report completion or failure. Follow
up load/unload queueing is then emitted deterministically.

Failed chunks do not automatically retry. A desired failed chunk stays `Failed`
until the host calls explicit retry. This prevents persistent provider failures
from creating an unbounded retry loop.

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
}

pub struct StreamRequest {
    pub request_id: StreamRequestId,
    pub chunk_id: runen_spatial::ChunkId,
    pub kind: StreamRequestKind,
    pub priority: ChunkPriority,
}

pub enum ProviderEventKind {
    Started,
    Completed,
    Failed,
}

pub struct ProviderEvent {
    pub request_id: StreamRequestId,
    pub chunk_id: runen_spatial::ChunkId,
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

pub struct WorldStreamingEvent {
    pub chunk_id: runen_spatial::ChunkId,
    pub request_id: Option<StreamRequestId>,
    pub kind: WorldStreamingEventKind,
}
```

Events must be deterministic and ordered by stable request priority, chunk id,
and request id.

`WorldStreamingController::tick` is fallible for checked spatial math. A focus
must use the controller's `WorldId`; a spatial failure is reported before the
controller changes lifecycle records or queues. Broader lifecycle hardening,
including checked request IDs, remains RS5 work.
