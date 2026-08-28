# RunenSpatial Architecture

This file owns the current package, dependency, and repository ownership structure. Conceptual spatial semantics live in the focused documents linked from [documentation architecture](docs/documentation-architecture.md). Future sequencing belongs in [ROADMAP.md](ROADMAP.md).

## Framework boundary

RunenSpatial owns host-neutral spatial mechanics that can be defined and tested without an engine, renderer, ECS, product system, IO backend, network stack, or application runtime.

It does not own world generation, SDF/product semantics, gameplay/simulation activation, persistence formats, replication authority, render resources, async execution, payload caches, or application retry/degradation policy.

## Workspace packages

### `runen-spatial`

Foundation package. Owns world-qualified identities, global/frame-local positions, grid partitioning, hierarchy math, clipmap/ring mapping, bounds, checked spatial arithmetic, and deterministic spatial-hash primitives.

It has no RunenSpatial runtime-package dependency.

### `runen-spatial-index`

Current provisional generic spatial-index package. It depends on `runen-spatial`.

The package's existence is not a commitment to retain or publish it. Its mutation semantics, extreme-coordinate behavior, complexity bounds, and independent consumer value require a dedicated decision before it becomes a durable boundary.

### `runen-spatial-demand`

Current spatial-demand package. It depends on `runen-spatial` and currently exposes the inherited single-focus/hysteresis planner.

Demand owns geometry-to-demand calculation and deterministic demand ordering. It does not own loading, IO, payloads, activation, or product purpose.

### `runen-spatial-streaming`

Content-agnostic availability controller. It depends on `runen-spatial` and `runen-spatial-demand`.

It owns request correlation, budgeted load/unload progression, lifecycle/event mechanics, and diagnostics that are independent of the backend payload. Backend IO, resource ownership, retries/timeouts beyond explicit framework contracts, and host activation remain outside.

## Non-core workspace members

### `godot_world_streaming`

Optional, non-default, non-publishable experimental adapter over the framework packages plus Godot. It is not framework authority and must not leak Godot semantics into core packages.

### `chunk_streaming_demo`

Repository-local executable demonstrating the current public framework flow. It is not an API or conformance authority.

### `xtask`

Private repository tooling behind `cargo validate`. It is not part of the runtime package surface.

## Dependency direction

```text
runen-spatial
├── runen-spatial-index
├── runen-spatial-demand
│   └── runen-spatial-streaming
├── chunk_streaming_demo      (consumer)
└── godot_world_streaming     (optional consumer)
```

`runen-spatial-streaming` also depends directly on `runen-spatial`.

Core packages must remain independent of Runenwerk, Godot, SDF/product implementations, ECS, rendering/GPU code, filesystem/network IO, and application runtimes.

## External integration

Runenwerk still has its own internal spatial/index/chunking/world-streaming implementation and does not currently depend on this workspace. A future consumer cutover is separate downstream work: prove the standalone contract first, migrate one accepted component at a time, then delete the corresponding duplicate Runenwerk authority without forwarding crates or mirrored source.
