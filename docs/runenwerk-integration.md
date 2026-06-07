# Runenwerk Integration

Runenwerk feature integration is deferred until the Godot world lab proves the
streaming and visual slice end to end. The existing Runenwerk wrapper migration
is a compatibility bridge only; it is not the start of product/runtime
integration.

## Migration Status

1. `domain/spatial` remains the Runenwerk dependency name and re-exports
   `spatial_streaming/crates/spatial`.
2. `domain/spatial_index` remains the Runenwerk dependency name and re-exports
   `spatial_streaming/crates/spatial_index`.
3. `domain/chunking` remains the Runenwerk dependency name and re-exports
   `spatial_streaming/crates/chunking`.
4. The wrapper packages are versioned separately from the extracted packages so
   Cargo can represent both packages in one lockfile during the transition.
5. `godot_chunking_addon` still compiles against the wrappers. Replacing it
   with `godot_world_streaming` is deferred until the Godot lab proves the
   streaming-only, generated-grid, visual-descriptor, MultiMesh, unload, cache,
   and dirty-update path.

## Deferred Feature Integration

Do not bring the Godot lab result back into Runenwerk until the lab has proven:

- streaming debug boxes;
- deterministic chunk-local generation;
- descriptor conversion through `godot_grid`;
- custom MultiMesh chunk visuals;
- unload and pooling behavior;
- host-owned async provider behavior;
- optional cache/save behavior if it exists;
- dirty-cell incremental updates.

Until then, Runenwerk should only consume the compatibility wrappers needed to
avoid duplicate local spatial/chunking implementations.

## Remaining Cleanup

Remove duplicated Runenwerk source modules only after the wrapper migration is
accepted and downstream consumers no longer need local fallback source history.

## Reference-Only Sources

The `world_streaming` implementation may read Runenwerk engine lifecycle code
for context, but must not mechanically extract from it. These concepts stay in
Runenwerk:

- dirty reasons;
- chunk revisions and build generations;
- world build queues;
- SDF payload formation;
- render cache invalidation;
- ECS resources and systems;
- networking replication state.

## Validation Before Future Integration

- `cargo test` in `spatial_streaming`.
- Runenwerk checks for current spatial/chunking consumers.
- Adapter checks only after the Godot adapter exists.
- No migration step should widen `spatial_streaming` to absorb `world_ops`,
  `world_sdf`, `procgen`, `product`, engine, editor, or apps.
