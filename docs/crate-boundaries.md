# Crate Boundaries

| Crate | Owns | Must not own |
| --- | --- | --- |
| `spatial` | World ids, world/local positions, chunk and region coordinates, hierarchy, grid partitioning, clipmap windows, ring-buffer mapping, minimal reusable spatial bounds DTOs, payload-neutral spatial hash helpers. | Residency decisions, streaming lifecycle, generation rules, IO, ECS, SDF payloads, rendering, editor behavior. |
| `spatial_index` | Spatial lookup and indexing over stable keys and `spatial::SpatialAabb3`. | Geometry vocabulary, chunk policy, collision response, ECS resources, world edit invalidation. |
| `chunking` | Desired chunk residency math: focus, load/unload radii, planar or 3D mode, chunk sets, diffs, deterministic ordering. | Loading, unloading, lifecycle states, provider IO, payload ownership. |
| `world_streaming` | Chunk lifecycle states, stream requests, provider events, deterministic lifecycle events, budgets, priorities, resident failure reporting. | SDF payloads, ECS spawning, asset catalogs, renderer resources, Godot nodes, save formats, mesh generation. |
| `world_core_prelude` | Focused common re-exports for normal workflows. The name preserves the original extraction target, but the crate exports only spatial streaming core APIs. | Advanced ownership, adapter APIs, runtime-only details, world payload semantics. |
| Optional `godot_world_streaming` | Godot position/config translation, ticking the core controller, Godot-friendly lifecycle signals. | Core policy, visuals, tile topology, asset lookup, scene ownership. |

## Anti-Creep Rules

- A crate may consume `spatial::ChunkId`; that does not make its payload logic
  part of this repository.
- `world_ops` remains the owner of world edits, dirty regions, build queues, and
  replication deltas.
- `world_sdf` remains the owner of SDF chunk/page payloads, field products,
  collision readiness, and cave/sector summaries.
- `procgen` remains the owner of procedural documents and lowering policy.
- `product` remains the owner of formed product descriptors and publication
  contracts.
- Godot adapters translate; they do not define reusable invariants.
