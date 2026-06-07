# Roadmap

This roadmap is intentionally conservative. The biggest long-term risk is
building too much too soon and letting a Godot scene, GridMap, MeshLibrary, or
Runenwerk runtime detail become the new source of truth.

The source-of-truth split is:

```text
Crystonix/grid
  owns local grid math, storage, dual-grid masks, descriptors, dirty-cell
  invalidation, and optional descriptor adapters.

Crystonix/spatial_streaming
  owns world/chunk coordinates, spatial indexing, desired chunk residency,
  request/event lifecycle control, and optional streaming adapters.

Crystonix/godot_world_lab
  owns the Godot project, generation experiments, mesh assets, materials,
  TileMeshCatalog, ChunkVisualBuilder, scenes, debug UI, and visual realization.

Runenwerk
  keeps only compatibility wrapper consumption for now; feature integration
  waits for the Godot proof and keeps SDF, product, procgen, editor, engine,
  apps, and platform semantics.
```

The operating rule:

```text
grid decides what tiles are needed.
spatial_streaming decides when chunks live.
Godot decides how chunks become visible.
Runenwerk later decides what chunks mean in the full platform, after the Godot
proof has validated streaming and visual realization independently.
```

## Critical Review

The current repository boundaries are correct. The remaining long-term risk is
scope creep: Godot can become too authoritative over topology, or
`world_streaming` can silently absorb provider, async, cache, or save/load
concerns.

M0 establishes the required lifecycle baseline:

1. Provider work is non-cancellable until a complete cancellation contract is
   designed.
2. Failed chunks stay failed until explicit retry or cleanup.
3. Streaming events carry optional request identity so host traces can
   correlate provider work with lifecycle transitions.

The Godot path must stay a realization layer. It may own scenes, meshes,
materials, chunk roots, async provider experiments, cache prototypes, and debug
UI, but it must not become the reusable topology or streaming policy source.

Save/load and async execution need explicit ownership. `world_streaming` remains
a synchronous deterministic state machine. Hosts may load, cache, save, await,
thread, or defer work, but those mechanisms must not leak into core lifecycle
policy.

The revised sequence below treats correctness gates as blockers. Later
milestones should not compensate for weak lifecycle behavior in Godot scripts.

## Milestone Summary

```text
M0 - Harden world_streaming lifecycle correctness
M1 - Keep grid descriptor/catalog boundary clean
M2 - Create Crystonix/godot_world_lab
M3 - Prove streaming with debug chunk boxes
M4 - Add deterministic Godot-side chunk generation
M5 - Convert generated grids to visual descriptors
M6 - Build MultiMesh chunk visuals
M7 - Prove unload, pooling, and budget pressure
M8 - Add host-owned async provider and optional chunk cache
M9 - Add dirty-cell incremental updates
M10 - Add mesh/material/catalog variants
M11 - Add optional ArrayMesh backend
M12 - Integrate with Runenwerk only after the Godot proof is complete
```

## M0 - Harden World Streaming

Status: complete. This milestone is the baseline for M1+ work.

Owner: `Crystonix/spatial_streaming`

Focus files:

```text
crates/world_streaming/src/controller.rs
crates/world_streaming/src/events.rs
crates/world_streaming/src/request.rs
crates/world_streaming/src/error.rs
crates/world_streaming/tests/controller.rs
adapters/godot_world_streaming/src/world_streaming_node.rs
docs/streaming-lifecycle.md
```

Preferred lifecycle decision for M0:

```text
Provider work is non-cancellable for now.
```

Reason: real cancellation needs a provider contract, task ownership semantics,
adapter behavior, and host guarantees. Advertising cancellation without those
contracts is worse than not supporting cancellation. Add cancellation later only
as a complete contract.

Implemented contract:

- Request kinds are load and unload only.
- Remove unconditional failed-record requeue behavior.
- Add explicit retry API, for example
  `WorldStreamingController::retry_failed_chunk(chunk_id)`.
- Keep `Failed` chunks in `Failed` until explicit retry or until they become
  undesired and can be cleaned up.
- Extend `WorldStreamingEvent` with
  `request_id: Option<StreamRequestId>`.
- Update event creation so request-related events carry the active request id.
- Keep provider completion deterministic when a chunk became undesired while
  load work was active: completion reaches `Resident`, then queues unload in a
  stable event order.
- Keep provider completion deterministic when a chunk became desired while
  unload work was active: completion reaches `Absent`, then queues load in a
  stable event order.
- Update the Godot adapter to use event request ids instead of side-channel
  correlation where possible.

Required tests:

- Chunk exits desired set while `LoadRequested`.
- Chunk exits desired set while `Loading`.
- Chunk becomes desired again while `UnloadRequested`.
- Chunk becomes desired again while `Unloading`.
- Provider failure does not auto-retry forever.
- Explicit retry of a failed desired chunk works.
- Provider completion after undesired load queues unload deterministically.
- Event ordering remains deterministic across identical inputs.
- Godot adapter still maps lifecycle-specific signals after event shape changes.

Exit gate:

```text
cargo fmt --all
cargo test
cargo check --workspace
cargo check -p godot_world_streaming
cargo run -p chunk_streaming_demo
```

No M1+ work should compensate for missing M0 semantics.

## Core Save/Load and Async Boundary

This boundary applies to every milestone:

```text
world_streaming is synchronous and deterministic.
```

Core input/output remains:

```text
tick(focus) -> StreamRequest[]
accept_provider_event(event) -> WorldStreamingEvent[]
```

The core must not own:

```text
async runtimes
threads
channels
filesystem IO
network IO
save formats
chunk payload persistence
provider task handles
Godot nodes
asset loading
```

Streaming lifecycle state is runtime state, not save state. Hosts save chunk
content or generation inputs; they do not save pending request ids, `Loading`
state, Godot nodes, MultiMesh instances, or controller internals.

Cancellation remains out of scope until a future provider contract defines task
ownership, completion guarantees, and request identity behavior across async
work.

## M1 - Keep Grid Descriptor Boundary Clean

Owner: `Crystonix/grid`

Current hardcoded descriptor keys such as `corner_90`, `edge_180`, and `full`
are acceptable for the first vertical slice. They are stable descriptors, not
asset ownership.

Later addition:

```text
GodotTileAssetCatalog
GodotTileAssetResolver
```

Purpose:

```text
VisualTileKind + rotation + variant -> asset key / mesh key / material key
```

Hard boundary:

```text
godot_grid must not own Mesh, PackedScene, MultiMesh, scene roots, materials,
generation policy, streaming state, or chunk lifecycle.
```

Exit gate:

```text
cargo fmt --all
cargo test
cargo test -p godot_grid
cargo build -p godot_grid
```

## M2 - Create Godot World Lab

Owner: `Crystonix/godot_world_lab`

Create a separate Godot project repository:

```text
Crystonix/godot_world_lab
  project.godot
  addons/
    godot_grid/
    godot_world_streaming/
  scenes/
    main.tscn
    chunk_root.tscn
  scripts/
    world_controller.gd
    chunk_provider.gd
    chunk_visual_builder.gd
    tile_mesh_catalog.gd
    chunk_debug_overlay.gd
  assets/
    tiles/
      corner.glb
      edge.glb
      t.glb
      diagonal.glb
      full.glb
      debug.glb
  docs/
    architecture.md
    chunk-visual-pipeline.md
    mesh-authoring-guide.md
```

The lab owns Godot-specific realization and experiments. It does not define
reusable topology, streaming lifecycle, or Runenwerk platform meaning.

Exit gate:

```text
Godot project opens cleanly.
Both native addons load.
Docs state that GridMap and MeshLibrary are optional debug/prototype tools, not
runtime truth.
```

## M3 - Streaming Debug Boxes

Use `godot_world_streaming` before any tile visuals.

Scene shape:

```text
Main
  PlayerOrCamera
  WorldStreamingNode
  ChunkRootContainer
```

Files:

```text
scripts/world_controller.gd
scripts/chunk_provider.gd
scripts/chunk_debug_overlay.gd
```

Responsibilities:

```text
world_controller.gd
  calls update_focus_from_vector3(player position)
  listens to lifecycle signals
  coordinates chunk roots

chunk_provider.gd
  receives chunk_load_requested
  reports provider_started/provider_completed/provider_failed
  tracks pending request ids

chunk_debug_overlay.gd
  renders debug chunk boxes only
```

Done when:

```text
Moving the camera streams debug chunk boxes in and out.
No duplicate chunk roots exist.
No orphan chunk roots remain after unload.
Pending request count stays bounded during fast movement.
Provider request ids match lifecycle events.
```

## M4 - Deterministic Godot-Side Generation

Keep generation in the Godot lab until a reusable pattern is proven.

File:

```text
scripts/chunk_provider.gd
```

Method:

```text
generate_chunk_logic_grid(chunk_coord)
```

Rules:

```text
Same chunk coordinate always generates the same grid.
Different chunk coordinates generate different layouts.
Generation is independent from Runenwerk.
No procgen graph or SDF concepts enter the lab slice.
```

## M5 - Grid Descriptor Conversion

Use `godot_grid` to convert generated chunk-local logic grids into visual
plans.

File:

```text
scripts/chunk_visual_builder.gd
```

Method:

```text
build_visual_plan(chunk_coord, logic_grid)
```

Flow:

```text
logic grid
  -> godot_grid / tile_topology
  -> VisualTileData[]
  -> skip empty tiles
  -> group by asset_key
```

Done when:

```text
One generated chunk produces stable visual tile descriptors.
Asset keys are stable.
Rotations are correct.
Empty tiles are skipped.
Chunk-local coordinate conventions are documented.
```

## M6 - MultiMesh Chunk Visual Builder

Do not make `GridMap` or `MeshLibrary` the runtime architecture.

Use:

```text
TileMeshCatalog
ChunkVisualBuilder
ChunkRoot
MultiMeshInstance3D buckets
```

Target runtime shape:

```text
ChunkRoot
  MultiMeshInstance3D corner
  MultiMeshInstance3D edge
  MultiMeshInstance3D t
  MultiMeshInstance3D diagonal
  MultiMeshInstance3D full
```

Mesh mapping:

```text
corner_90   -> corner mesh + 90 degree rotation
edge_180    -> edge mesh + 180 degree rotation
t_0         -> t mesh
diagonal_90 -> diagonal mesh + 90 degree rotation
full        -> full mesh
debug       -> debug mesh
```

Done when:

```text
Resident chunk creates a visible MultiMesh chunk root.
Unloaded chunk removes or pools its root.
Chunk origin is correct.
Visuals align across chunk borders.
The full chunk rebuild path remains available for debugging.
```

## M7 - Unload, Pooling, and Budget Pressure

Stress the lifecycle before adding incremental editing complexity.

Scenarios:

```text
Fast camera movement.
Repeated load/unload around radius boundaries.
Provider start/completion delay.
Provider failure.
Chunk root pooling and reuse.
```

Done when:

```text
Pending request count stays bounded.
No chunk roots leak.
No pooled root is reused with stale transforms or stale meshes.
Unload completion is reported back to spatial_streaming.
```

## M8 - Host-Owned Async Provider and Optional Chunk Cache

Add async and cache behavior in the Godot lab, not in `world_streaming`.

Files:

```text
scripts/chunk_provider.gd
scripts/chunk_cache.gd
docs/chunk-cache-and-provider.md
```

Provider rule:

```text
The provider may defer, await, thread, or simulate work, but it reports only
ProviderEvent values back to godot_world_streaming.
```

Optional cache/save scope:

```text
chunk_coord
generator_version
logic_grid cells
dirty overrides
catalog or theme id
```

Explicitly do not cache/save:

```text
WorldStreamingController internals
pending request ids
LoadRequested / Loading / UnloadRequested / Unloading state
Godot nodes
ChunkRoot instances
MultiMesh instances
renderer resources
```

Done when:

```text
Delayed provider completion still produces deterministic lifecycle results.
Fast movement keeps pending work bounded.
Cached chunks reload content without restoring runtime lifecycle state.
Cache invalidation handles generator_version changes.
All save/cache behavior remains Godot-lab-owned.
```

## M9 - Dirty-Cell Incremental Updates

Add dirty updates only after full chunk build and unload are proven.

File:

```text
scripts/chunk_visual_builder.gd
```

Methods:

```text
update_dirty_cell(chunk_root, logic_grid, cell_coord)
update_visual_tiles(chunk_root, visual_tile_data_array)
```

Use `godot_grid` dirty helpers for affected visual corners.

Done when:

```text
Changing one logic cell updates exactly four visual corners.
Full chunk rebuild remains available as a debug fallback.
Incremental updates do not desynchronize MultiMesh buckets.
```

## M10 - Mesh, Material, and Catalog Variants

Add variants only after the base builder is stable.

Examples:

```text
theme or biome variant
material key
debug material
missing asset fallback
catalog validation report
```

Done when:

```text
The catalog validates all descriptor keys used by generated chunks.
Missing mappings fail visibly in debug builds.
Variant selection does not affect grid descriptor truth.
```

## M11 - Optional ArrayMesh Backend

Do not start with merged chunk meshes. Start with MultiMesh and add ArrayMesh
only when profiling shows the need.

Stable interface:

```text
ChunkVisualBuilder
  build_chunk_visual(chunk_coord, visual_tiles, catalog)
  update_dirty_tiles(handle, changed_tiles)
  destroy_or_pool(handle)
```

Backends:

```text
MultiMeshChunkVisualBuilder
  first production backend

ArrayMeshChunkVisualBuilder
  later optimization backend

GridMapChunkVisualBuilder
  optional prototype/debug backend only
```

Done when:

```text
ArrayMesh backend matches MultiMesh visual output.
Backend selection does not change streaming, topology, or catalog truth.
Dirty updates either work correctly or fall back to full backend rebuild.
```

## M12 - Runenwerk Integration

Runenwerk feature integration is explicitly deferred. Compatibility wrappers may
remain so Runenwerk can consume the extracted `spatial`, `spatial_index`, and
`chunking` crates, but no new Runenwerk feature work should depend on the Godot
path until the Godot lab proves:

```text
streaming
generation
visual descriptors
custom chunk visuals
unload and pooling
host-owned async provider behavior
optional cache/save behavior if it exists
dirty updates
```

After that proof, Runenwerk consumes:

```text
Crystonix/grid
Crystonix/spatial_streaming
```

Runenwerk keeps:

```text
world_ops
world_sdf
product
procgen
editor
engine
apps
```

Done when:

```text
Godot lab evidence exists for M3 through M9.
Runenwerk integration does not move SDF, ECS, product, procgen, renderer,
editor, save, or app semantics into grid, spatial_streaming, or the Godot lab.
```

## Hard Non-Goals Before M12

```text
No Runenwerk feature integration.
No SDF.
No ECS.
No platform save/load.
No networking.
No procgen graph.
No editor tooling.
No true merged ArrayMesh first.
No GridMap or MeshLibrary as runtime source of truth.
No Godot scripts as reusable topology or streaming policy truth.
```

## Risk Register

| Risk | Consequence | Mitigation |
| --- | --- | --- |
| Fake cancellation semantics in core | Providers and adapters drift into incompatible behavior. | Remove cancellation until a full provider cancellation contract exists. |
| Automatic retry loops | Failed chunks churn forever under persistent failure. | Failed chunks stay failed until explicit retry or cleanup. |
| Missing request identity in events | Adapter traces and debugging become ambiguous. | Carry `Option<StreamRequestId>` on `WorldStreamingEvent`. |
| Async leaks into core | Determinism and host portability collapse around runtime choices. | Keep async execution host-owned; core accepts only provider events. |
| Save/load leaks into core | Runtime request state hardens into a bad persistence format. | Save chunk content or generation inputs in the host, never controller internals. |
| GridMap becomes architecture | Godot scene state becomes topology truth. | Keep GridMap as debug/prototype backend only. |
| Mesh catalog leaks into `grid` | Descriptor crate becomes asset/runtime specific. | Keep catalog in Godot lab; `godot_grid` maps descriptors to value keys only. |
| Generation moves to reusable crates too early | Prototype policy hardens into bad public API. | Keep generation in Godot lab until stable reusable rules emerge. |
| Runenwerk integration starts early | Platform-specific SDF/product/editor semantics leak back into reusable crates. | Keep only compatibility wrappers before Godot proof; block feature integration until M3-M9 are proven independently. |

## Immediate Next Work

Start with M0.

Implementation prompt:

```text
Review and harden Crystonix/spatial_streaming/crates/world_streaming.

Focus files:
- crates/world_streaming/src/controller.rs
- crates/world_streaming/src/events.rs
- crates/world_streaming/src/request.rs
- crates/world_streaming/src/error.rs
- crates/world_streaming/tests/controller.rs
- adapters/godot_world_streaming/src/world_streaming_node.rs
- docs/streaming-lifecycle.md

Required corrections:
1. Remove cancellation request kinds for now and document providers as
   non-cancellable.
2. Stop automatic failed-chunk retry loops.
3. Add explicit retry_failed_chunk(chunk_id).
4. Add request_id: Option<StreamRequestId> to WorldStreamingEvent.
5. Add lifecycle tests for desired-state reversals, failure retry, undesired
   completion unload queueing, and deterministic event order.
6. Do not introduce IO, async runtime, Godot nodes, ECS, assets, SDF, renderer
   state, Runenwerk concepts, or provider ownership.

Return changed files, tests added, and exact validation commands.
```
