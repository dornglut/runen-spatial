# RunenSpatial Architecture

This file owns the current package, dependency, and repository ownership structure. Conceptual spatial semantics live in the focused documents linked from [documentation architecture](docs/documentation-architecture.md). Future sequencing belongs in [ROADMAP.md](ROADMAP.md).

## Framework boundary

RunenSpatial owns host-neutral spatial mechanics that can be defined and tested without an engine, renderer, ECS, product system, IO backend, network stack, or application runtime.

It does not own world generation, SDF/product semantics, gameplay/simulation activation, persistence formats, replication authority, render resources, async execution, payload caches, or application retry/degradation policy.

## Workspace packages

### `runen-spatial`

Foundation package. Owns world-qualified identities, global/frame-local positions, grid partitioning, hierarchy math, clipmap/ring mapping, checked spatial arithmetic, and deterministic spatial-hash primitives.

It has no RunenSpatial runtime-package dependency.

No generic spatial-index package is retained. The inherited provisional index had no maintained consumer and did not justify an independent package contract. A future index belongs with a proven consumer until reusable ownership and complexity requirements are established.

### `runen-spatial-demand`

Spatial-demand package. It depends on `runen-spatial` and owns one world-bound, bounded deterministic multi-source planner with validated horizontal/vertical box demand, source-local hysteresis, explicit pins, effective ranks, pressure, and deterministic deltas.

Demand does not own loading, IO, payloads, activation, product purpose, host source priority, or engine-specific geometry.

### `runen-spatial-streaming`

Content-agnostic availability controller. It depends on `runen-spatial` and `runen-spatial-demand`.

It owns request correlation, budgeted load/unload progression, lifecycle/event mechanics, and diagnostics that are independent of the backend payload. Backend IO, resource ownership, retries/timeouts beyond explicit framework contracts, and host activation remain outside.

## Non-core workspace members

### `godot_world_streaming`

Retained optional, non-default, non-publishable pre-release translation adapter over the framework packages plus Godot. Maintained consumer evidence justifies the integration artifact, but it is not framework authority and must not leak Godot semantics into core packages.

### `xtask`

Private repository tooling behind `cargo validate`. It is not part of the runtime package surface.

## Dependency direction

```text
runen-spatial
└── runen-spatial-demand
    └── runen-spatial-streaming

godot_world_streaming        (retained optional integration)
├── runen-spatial
├── runen-spatial-demand
└── runen-spatial-streaming
```

`runen-spatial-streaming` also depends directly on `runen-spatial`.

Core packages must remain independent of Runenwerk, Godot, SDF/product implementations, ECS, rendering/GPU code, filesystem/network IO, and application runtimes.

## External integration

Runenwerk still has its own internal spatial/index/chunking/world-streaming implementation and does not currently depend on this workspace. A future consumer cutover is separate downstream work: prove the standalone contract first, migrate one accepted component at a time, then delete the corresponding duplicate Runenwerk authority without forwarding crates or mirrored source.
