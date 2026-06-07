# Spatial Streaming

Reusable Rust crates for spatial addressing, spatial indexing, desired chunk
residency, and payload-neutral chunk streaming lifecycle control.

This repository is intentionally narrower than a world-core or terrain stack.
It is not the world model. It does not own world edits, SDF payloads, procgen,
assets, ECS spawning, renderer resources, replication, save formats, or Godot
scene ownership.

## Workspace

The default workspace build includes only engine-neutral core crates:

- `crates/spatial`
- `crates/spatial_index`
- `crates/chunking`
- `crates/world_streaming`
- `crates/world_core_prelude`

Optional members are present but opt-in:

- `adapters/godot_world_streaming`
- `demos/chunk_streaming_demo`

`godot_world_streaming` is adapter-only, and the demo shows how hosts compose
streaming events with `Crystonix/grid` topology descriptors.

## Core Usage

```rust
use world_core_prelude::*;

let partition = GridPartitionConfig {
    chunk_edge_meters: 16.0,
    ..GridPartitionConfig::default()
};

let config = WorldStreamingConfig::new(
    WorldId(1),
    partition,
    ChunkStreamingConfig::default(),
);
let mut controller = WorldStreamingController::new(config);
let output = controller.tick(StreamingTick::from_focus(StreamingFocus::new([0.0, 0.0, 0.0])));

assert!(!output.requests.is_empty());
```

Run the core validation surface:

```text
cargo test
```

Run opt-in adapter/demo validation explicitly:

```text
cargo check -p godot_world_streaming
cargo run -p chunk_streaming_demo
```

## Staging

1. Stage 1 complete: spatial, spatial_index, chunking, world_streaming, prelude, and docs.
2. Stage 2 complete: `godot_world_streaming` adapter-only Godot extension.
3. Stage 3 complete: chunk streaming demo composing lifecycle events with `Crystonix/grid`.
4. Stage 4 complete: Runenwerk compatibility wrappers point at these extracted crates.

## Roadmap

See [docs/roadmap.md](docs/roadmap.md) for the cross-repository milestone plan.
The next required milestone is M0: harden `world_streaming` lifecycle
correctness before serious Godot world work.

For an executable `/goal` prompt, use
[docs/full-roadmap-goal-prompt.md](docs/full-roadmap-goal-prompt.md).
