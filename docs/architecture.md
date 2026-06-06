# Architecture

`spatial_streaming` is spatial addressing plus chunk residency and streaming
lifecycle contracts only.

It is not the world model.

It is not a terrain stack.

It is not an engine runtime.

The repository name is intentionally stricter than `world_core` to reduce future
creep. A reusable world may consume these crates, but this repository must not
become the owner of world operations, field payloads, procedural generation,
assets, ECS runtime state, networking replication, save formats, renderer
resources, or editor behavior.

## Current Tree

```text
spatial_streaming/
  Cargo.toml
  README.md
  crates/
    spatial/
    spatial_index/
    chunking/
    world_streaming/
    world_core_prelude/
  adapters/
    godot_world_streaming/
  demos/
    chunk_streaming_demo/
  docs/
    architecture.md
    crate-boundaries.md
    spatial-model.md
    chunking-model.md
    streaming-lifecycle.md
    godot-integration.md
    grid-composition.md
    runenwerk-integration.md
```

The adapter and demo are workspace members but intentionally outside
`default-members`, so the normal build surface remains the reusable core.

## Dependency Direction

```text
spatial
spatial_index -> spatial
chunking -> spatial
world_streaming -> spatial, chunking
world_core_prelude -> spatial, spatial_index, chunking, world_streaming

optional:
godot_world_streaming -> godot, spatial, chunking, world_streaming
chunk_streaming_demo -> world_core_prelude, Crystonix/grid tile_topology
```

No core crate may depend on Godot, Runenwerk engine crates, ECS runtime crates,
`world_ops`, `world_sdf`, `procgen`, `product`, rendering, editor, or apps.

## Provider Ownership Model

The `world_streaming` crate uses request/event ownership:

1. `world_streaming` emits `StreamRequest`.
2. The host, adapter, engine, or app performs loading/unloading.
3. The host reports `ProviderEvent` back.
4. `world_streaming` advances lifecycle state and emits deterministic events.

The core must not call provider code directly in the first extraction. It must
not own async runtimes, worker threads, filesystem IO, asset catalogs, Godot
nodes, renderer uploads, mesh generation, or SDF payload loading.
