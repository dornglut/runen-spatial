# Full Roadmap Goal Prompt

Use this as a `/goal` prompt when the intent is to execute the full roadmap, not
just discuss it.

```text
Execute the cross-repository world lab roadmap in
/Users/joshua/Projekte/spatial_streaming/docs/roadmap.md from M0 through M12.

Work step by step, in order, until the roadmap is complete or a real blocker
prevents progress. Do not skip gates, weaken boundaries, or compensate for
unfinished lifecycle semantics in Godot scripts. Make coherent scoped commits
after each completed milestone or coherent sub-milestone. Do not push unless I
explicitly ask.

Source-of-truth split:
- Crystonix/grid owns grid math/storage/topology/descriptors/dirty-cell
  invalidation and optional descriptor adapters.
- Crystonix/spatial_streaming owns coordinates, spatial indexing, desired
  residency, and payload-neutral request/event lifecycle control.
- Crystonix/godot_world_lab owns Godot project realization: generation
  experiments, mesh assets, materials, TileMeshCatalog, ChunkVisualBuilder,
  MultiMesh/ArrayMesh backends, scenes, debug UI, async provider behavior, and
  optional chunk cache/save prototypes.
- Runenwerk keeps SDF/product/procgen/editor/engine/apps/platform meaning.

Hard order:
M0 harden world_streaming lifecycle correctness first.
M1 keep grid descriptor/catalog boundary clean.
M2 create Crystonix/godot_world_lab.
M3 prove streaming with debug chunk boxes.
M4 add deterministic Godot-side chunk generation.
M5 convert generated grids to visual descriptors through godot_grid.
M6 build MultiMesh chunk visuals.
M7 prove unload, pooling, and budget pressure.
M8 add host-owned async provider and optional chunk cache.
M9 add dirty-cell incremental updates.
M10 add mesh/material/catalog variants.
M11 add optional ArrayMesh backend.
M12 integrate with Runenwerk only after Godot proof evidence exists.

M0 requirements:
- Remove fake cancellation request kinds unless a complete provider
  cancellation contract is implemented. Prefer non-cancellable providers now.
- Stop automatic failed-chunk retry loops.
- Add explicit retry_failed_chunk(chunk_id).
- Add request_id: Option<StreamRequestId> to WorldStreamingEvent.
- Add lifecycle tests for desired-state reversals, failure retry, undesired
  completion unload queueing, and deterministic event order.
- Update godot_world_streaming and docs accordingly.

Never add IO, async runtime, threads, channels, save formats, Godot nodes, ECS,
assets, SDF, renderer state, Runenwerk concepts, or provider ownership to core
spatial_streaming crates. world_streaming must remain a synchronous
deterministic state machine: tick -> StreamRequest[], provider reports
ProviderEvent, controller emits WorldStreamingEvent[].

Do not use GridMap or MeshLibrary as runtime source of truth. They may exist
only as debug/prototype backends. Godot realizes chunks; it does not define
topology or streaming truth.

Do not do Runenwerk feature integration before the Godot lab proves M3-M9.
Existing Runenwerk wrappers are compatibility-only.

For every milestone:
- read applicable AGENTS.md first;
- inspect existing code before editing;
- preserve unrelated user changes;
- run the milestone gate commands from roadmap.md;
- record changed files, tests, validation output, residual risk, and next step.
```
